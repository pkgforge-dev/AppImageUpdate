// system headers+
#include <algorithm>
#include <chrono>
#include <fnmatch.h>
#include <fstream>
#include <iostream>
#include <mutex>
#include <sstream>
#include <thread>

// library headers
#include <zsclient.h>
#include <cpr/cpr.h>

// local headers
#include "appimage/update.h"
#include "util.h"

// convenience declaration
typedef std::lock_guard<std::mutex> lock_guard;

namespace appimage {
    namespace update {
        class Updater::Private {
        public:
            Private() : state(INITIALIZED),
                            pathToAppImage(),
                            zSyncClient(nullptr),
                            thread(nullptr),
                            mutex() {};

            ~Private() {
                delete zSyncClient;
            }

        public:
            // data
            std::string pathToAppImage;

            // state
            State state;

            // ZSync client -- will be instantiated only if necessary
            zsync2::ZSyncClient* zSyncClient;

            // threading
            std::thread* thread;
            std::mutex mutex;

        public:
            enum UpdateInformationType {
                INVALID = -1,
                ZSYNC_GENERIC = 0,
                ZSYNC_GITHUB_RELEASES,
                ZSYNC_BINTRAY,
            };

            struct AppImage {
                std::string filename;
                int appImageVersion;
                UpdateInformationType updateInformationType;
                std::string zsyncUrl;

                AppImage() : appImageVersion(-1), updateInformationType(INVALID) {};
            };
            typedef struct AppImage AppImage;

        public:

            static const AppImage* readAppImage(const std::string pathToAppImage) {
                // error state: empty AppImage path
                if (pathToAppImage.empty())
                    return nullptr;

                // check whether file exists
                std::ifstream ifs(pathToAppImage);

                // if file can't be opened, it's an error
                if (!ifs || !ifs.good())
                    return nullptr;

                // read magic number
                ifs.seekg(8, std::ios::beg);
                unsigned char magicByte[4] = {0, 0, 0, 0};
                ifs.read((char*) magicByte, 3);

                // validate first two bytes are A and I
                if (magicByte[0] != 'A' || magicByte[1] != 'I')
                    return nullptr;

                uint8_t version;
                // the third byte contains the version
                switch (magicByte[2]) {
                    case '\x01':
                        version = 1;
                        break;
                    case '\x02':
                        version = 2;
                        break;
                    default:
                        return nullptr;
                }

                // read update information in the file
                std::string updateInformation;

                if (version == 1) {
//                    updateInformationCommand = "";
                } else if (version == 2) {
                    // check whether update information can be found inside the file by calling objdump
                    auto command = "objdump -h \"" + pathToAppImage + "\"";

                    std::string match;
                    if (!callProgramAndGrepForLine(command, ".upd_info", match))
                        return nullptr;

                    auto parts = split(match);
                    parts.erase(std::remove_if(parts.begin(), parts.end(),
                                               [](std::string s) {return s.length() <= 0;}
                    ));

                    auto offset = std::stoi(parts[5], nullptr, 16);
                    auto length = std::stoi(parts[2], nullptr, 16);

                    ifs.seekg(offset, std::ios::beg);
                    char rawUpdateInformation[2048] = {0};
                    ifs.read(rawUpdateInformation, length);

                    updateInformation = rawUpdateInformation;
                }

                UpdateInformationType uiType = INVALID;
                std::string zsyncUrl;

                // parse update information
                auto uiParts = split(updateInformation, '|');

                if (uiParts[0] == "gh-releases-zsync") {
                    // validate update information
                    if (uiParts.size() == 5) {
                        uiType = ZSYNC_GITHUB_RELEASES;

                        auto username = uiParts[1];
                        auto repository = uiParts[2];
                        auto tag = uiParts[3];
                        auto filename = uiParts[4];

                        std::stringstream url;
                        url << "https://api.github.com/repos/" << username << "/" << repository << "/releases/";

                        if (tag.find("latest") != std::string::npos) {
                            url << "latest";
                        } else {
                            url << "/tags/" << tag;
                        }

                        auto response = cpr::Get(url.str());

                        // continue only if HTTP status is good
                        if (response.status_code >= 200 && response.status_code < 300) {
                            // in contrary to the original implementation, instead of converting wildcards into
                            // all-matching regular expressions, we have the power of fnmatch() available, a real wildcard
                            // implementation
                            // unfortunately, this is still hoping for GitHub's JSON API to return a pretty printed
                            // response which can be parsed like this
                            std::stringstream responseText(response.text);
                            std::string currentLine;
                            auto pattern = "*" + filename + "*";
                            while (std::getline(responseText, currentLine)) {
                                if (currentLine.find("browser_download_url") != std::string::npos) {
                                    if (fnmatch(pattern.c_str(), currentLine.c_str(), 0) == 0) {
                                        auto parts = split(currentLine, '"');
                                        zsyncUrl = parts.back();
                                        uiType = ZSYNC_GITHUB_RELEASES;
                                    }
                                }
                            }
                        }
                    }
                } else if (uiParts[0] == "bintray-zsync") {
                    if (uiParts.size() == 5) {
                        auto username = uiParts[1];
                        auto repository = uiParts[2];
                        auto packageName = uiParts[3];
                        auto filename = uiParts[4];

                        std::stringstream downloadUrl;
                        downloadUrl << "https://dl.bintray.com/" << username << "/" << repository  << "/" << filename;

                        std::stringstream redirectorUrl;
                        redirectorUrl << "https://bintray.com/" << username << "/" << repository << "/"
                                      << packageName << "/_latestVersion";

                        auto versionResponse = cpr::Get(redirectorUrl.str());
                        // this request is supposed to be redirected
                        // due to how cpr works, we can't check for a redirection status, as we get the response for
                        // the redirected request
                        // therefore, we check for a 2xx response, and then can inspect and compare the redirected URL
                        if (versionResponse.status_code >= 200 && versionResponse.status_code < 400) {
                            auto redirectedUrl = versionResponse.url;

                            // if they're different, it's probably been successful
                            if (redirectorUrl.str() != redirectedUrl) {
                                // the last part will contain the current version
                                auto packageVersion = static_cast<std::string>(split(redirectedUrl, '/').back());
                                auto urlTemplate = downloadUrl.str();

                                // split by _latestVersion, insert correct value, compose final value
                                auto pos = urlTemplate.find("_latestVersion");
                                auto firstPart = urlTemplate.substr(0, pos);
                                auto secondPart = urlTemplate.substr(pos + std::string("_latestVersion").length());
                                zsyncUrl = firstPart + packageVersion + secondPart;
                                uiType = ZSYNC_BINTRAY;
                            }
                        }

                    }
                } else if (uiParts[0] == "zsync") {
                    // validate update information
                    if (uiParts.size() == 2) {
                        zsyncUrl = uiParts.back();
                        uiType = ZSYNC_GENERIC;
                    }
                } else {
                    // unknown type
                }

                auto* appImage = new AppImage();

                appImage->filename = pathToAppImage;
                appImage->appImageVersion = version;
                appImage->updateInformationType = uiType;
                appImage->zsyncUrl = zsyncUrl;

                return appImage;
            }
        };
        
        Updater::Updater(const char* pathToAppImage) {
            // initialize data class
            d = new Updater::Private();

            // check whether file exists, otherwise throw exception
            std::ifstream f(pathToAppImage);

            if(!f || !f.good())
                throw std::invalid_argument("No such file or directory: " + d->pathToAppImage);

            d->pathToAppImage = pathToAppImage;
        }

        Updater::~Updater() {
            delete d;
        }

        void Updater::runUpdate() {
            // initialization
            {
                lock_guard guard(d->mutex);

                // make sure it runs only once at a time
                // should never occur, but you never know
                if (d->state != INITIALIZED)
                    return;

                // WARNING: if you don't want to shoot yourself in the foot, make sure to read in the AppImage
                // while locking the mutex and/or before the RUNNING state to make sure readAppImage() finishes
                // before progress() and such can be called! Otherwise, progress() etc. will return an error state,
                // causing e.g., main(), to interrupt the thread and finish.
                auto* appImage = d->readAppImage(d->pathToAppImage);

                // check whether update information is available
                if (appImage->updateInformationType == d->INVALID) {
                    d->state = ERROR;
                    return;
                }

                if (appImage->updateInformationType == d->ZSYNC_GITHUB_RELEASES ||
                    appImage->updateInformationType == d->ZSYNC_BINTRAY ||
                    appImage->updateInformationType == d->ZSYNC_GENERIC) {
                    // doesn't matter which type it is exactly, they all work like the same
                    d->zSyncClient = new zsync2::ZSyncClient(appImage->zsyncUrl);
                } else {
                    // error unsupported type
                    d->state = ERROR;
                    return;
                }

                d->state = RUNNING;

                // cleanup
                delete appImage;
            }

            // keep state -- by default, an error (false) is assumed
            bool result = false;

            // run phase
            {
                // check whether it's a zsync operation
                if (d->zSyncClient != nullptr) {
                    result = d->zSyncClient->run();
                }
            }

            // end phase
            {
                lock_guard guard(d->mutex);

                if (result) {
                    d->state = SUCCESS;
                } else {
                    d->state = ERROR;
                }
            }
        }

        bool Updater::start() {
            // lock mutex
            lock_guard guard(d->mutex);

            // prevent multiple start calls
            if(d->state != INITIALIZED)
                return false;

            // if there's a thread managed by this class already, should not start another one and lose access to
            // this one
            if(d->thread)
                return false;

            // create thread
            d->thread = new std::thread(&Updater::runUpdate, this);

            return true;
        }

        bool Updater::isDone() {
            lock_guard guard(d->mutex);

            return d->state != INITIALIZED && d->state != RUNNING && d->state != STOPPING;
        }

        bool Updater::hasError() {
            lock_guard guard(d->mutex);

            return d->state == ERROR;
        }

        bool Updater::progress(double& progress) {
            lock_guard guard(d->mutex);

            if (d->state == INITIALIZED) {
                progress = 0;
                return true;
            } else if (d->state == SUCCESS || d->state == ERROR) {
                progress = 1;

                delete d->zSyncClient;
                d->zSyncClient = nullptr;

                return false;
            }

            if (d->zSyncClient != nullptr) {
                progress = d->zSyncClient->progress();
                return true;
            }

            return false;
        }

        bool Updater::stop() {
            throw std::runtime_error("not implemented");
        }

        bool Updater::nextStatusMessage(std::string& message) {
            // first, check own message queue
            // TODO: implement message queue, issue status messages

            // next, check zsync client for a message
            if (d->zSyncClient != nullptr) {
                std::string zsyncMessage;
                if (!d->zSyncClient->nextStatusMessage(zsyncMessage))
                    return false;
                // show that the message is coming from zsync2
                message = "zsync2: " + zsyncMessage;
                return true;
            }

            return false;
        }

        State Updater::state() {
            return d->state;
        }
    }
}
