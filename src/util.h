#pragma once

// system headers
#include <algorithm>
#include <climits>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>
#include <unistd.h>

#ifdef FLTK_UI
    #include <FL/Fl.H>
#endif
#ifdef QT_UI
    #include <QMessageBox>
#endif

// library includes
#include <zsutil.h>

namespace appimage {
    namespace update {
        static void removeNewlineCharacters(std::string& str) {
            str.erase(std::remove(str.begin(), str.end(), '\n'), str.end());
        }

        static inline bool ltrim(std::string& s, char to_trim = ' ') {
            // TODO: find more efficient way to check whether elements have been removed
            size_t initialLength = s.length();
            s.erase(s.begin(), std::find_if(s.begin(), s.end(), [to_trim](int ch) {
                return ch != to_trim;
            }));
            return s.length() < initialLength;
        }

        static inline bool rtrim(std::string& s, char to_trim = ' ') {
            // TODO: find more efficient way to check whether elements have been removed
            auto initialLength = s.length();
            s.erase(std::find_if(s.rbegin(), s.rend(), [to_trim](int ch) {
                return ch != to_trim;
            }).base(), s.end());
            return s.length() < initialLength;
        }

        static inline bool trim(std::string& s, char to_trim = ' ') {
            // returns true if either modifies s
            auto ltrim_result = ltrim(s, to_trim);
            return rtrim(s, to_trim) && ltrim_result;
        }

        static bool callProgramAndGrepForLine(const std::string& command, const std::string& pattern,
                                              std::string& output) {
            FILE *stream = popen(command.c_str(), "r");

            if (stream == nullptr)
                return false;

            char *line;
            size_t lineSize = 0;
            while(getline(&line, &lineSize, stream)) {
                // check whether line matches pattern
                std::string lineString = line;
                if (lineString.find(pattern) != std::string::npos) {
                    if (pclose(stream) != 0) {
                        free(line);
                        return false;
                    }
                    output = line;
                    removeNewlineCharacters(output);
                    return true;
                }
            }

            if (pclose(stream) != 0) {
                free(line);
                return false;
            }

            return false;
        }

        static std::vector<std::string> split(const std::string& s, char delim = ' ') {
            std::vector<std::string> result;

            std::stringstream ss(s);
            std::string item;

            while (std::getline(ss, item, delim)) {
                result.push_back(item);
            }

            return result;
        }

        static inline std::string toLower(std::string s) {
            std::transform(s.begin(), s.end(), s.begin(), [](unsigned char c) { return std::tolower(c); });
            return s;
        }

        static inline bool toLong(const std::string& str, long& retval, int base = 10) {
            char* end = nullptr;
            const auto* cstr = str.c_str();

            auto rv = std::strtol(cstr, &end, base);
            if (errno == ERANGE || cstr == end || retval > LONG_MAX || retval < LONG_MIN)
                return false;

            retval = rv;
            return true;
        }

        static inline bool isFile(const std::string& path) {
            std::ifstream ifs(path);
            return (bool) ifs && ifs.good();
        }

        static void runApp(const std::string& path) {
            // make executable
            mode_t newPerms;
            auto errCode = zsync2::getPerms(path, newPerms);

            if (errCode != 0) {
                std::ostringstream ss;
                ss << "Error calling stat(): " << strerror(errCode);
#ifdef FLTK_UI
                fl_message("%s", ss.str().c_str());
#endif
#ifdef QT_UI
                QMessageBox::critical(nullptr, "Error", QString::fromStdString(ss.str()), QMessageBox::Close);
#endif
                exit(1);
            }

            chmod(path.c_str(), newPerms | S_IXUSR);

            // full path to AppImage, required for execl
            char* realPathToAppImage;
            if ((realPathToAppImage = realpath(path.c_str(), nullptr)) == nullptr) {
                auto error = errno;
                std::ostringstream ss;
                ss << "Error resolving full path of AppImage: code " << error << ": " << strerror(error) << std::endl;
#ifdef FLTK_UI
                fl_message("%s", ss.str().c_str());
#endif
#ifdef QT_UI
                QMessageBox::critical(nullptr, "Error", QString::fromStdString(ss.str()), QMessageBox::Close);
#endif
                exit(1);
            }

            if (fork() == 0) {
                putenv(strdup("STARTED_BY_APPIMAGEUPDATE=1"));

                std::cerr << "Running " << realPathToAppImage << std::endl;

                // make sure to deactivate updater contained in the AppImage when running from AppImageUpdate
                execl(realPathToAppImage, realPathToAppImage, nullptr);

                // execle should never return, so if this code is reached, there must be an error
                auto error = errno;
                std::cerr << "Error executing AppImage " << realPathToAppImage << ": code " << error << ": "
                          << strerror(error) << std::endl;
                exit(1);
            }
        }

        // Reads an ELF file section and returns its contents.
        static std::string readElfSection(const std::string& filePath, const std::string& sectionName) {
            // first of all, check whether there is an objdump binary next to the current one (probably
            // the bundled one in the AppImages of AppImageUpdate), otherwise use the system wide
            std::string objdump;

            {
                // TODO: replace this Linux specific solution with something platform independent
                std::vector<char> buffer(4096);
                readlink("/proc/self/exe", buffer.data(), buffer.size());

                auto pathToBinary = std::string(buffer.data());
                auto slashPos = pathToBinary.find_last_of('/');
                auto bundledObjdump = pathToBinary.substr(0, slashPos) + "/objdump";

                if (isFile(bundledObjdump)) {
                    objdump = bundledObjdump;
                } else {
                    objdump = "objdump";
                }
            }

            auto command = objdump + " -h \"" + filePath + "\"";

            std::string match;
            if (!callProgramAndGrepForLine(command, sectionName, match))
                return "";

            auto parts = split(match);
            parts.erase(std::remove_if(parts.begin(), parts.end(),
                [](std::string s) { return s.length() <= 0; }
            ));

            auto offset = (unsigned long) std::stoi(parts[5], nullptr, 16);
            auto length = (unsigned long) std::stoi(parts[2], nullptr, 16);

            std::ifstream ifs(filePath);

            ifs.seekg(offset, std::ios::beg);
            std::vector<char> rawUpdateInformation(length, '\0');
            ifs.read(rawUpdateInformation.data(), length);

            return rawUpdateInformation.data();
        }
    }; // namespace update
} // namespace appimage
