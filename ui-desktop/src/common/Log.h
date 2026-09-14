#pragma once
#include <iostream>
#include <string_view>

namespace FluyerLog {
    inline void Info(std::string_view tag, std::string_view msg) {
        std::cout << "\033[1;36m[" << tag << "]\033[0m \033[32m" << msg << "\033[0m\n";
    }

    inline void Warn(std::string_view tag, std::string_view msg) {
        std::cout << "\033[1;33m[" << tag << "]\033[0m \033[33m" << msg << "\033[0m\n";
    }

    inline void Error(std::string_view tag, std::string_view msg) {
        std::cerr << "\033[1;31m[" << tag << "]\033[0m \033[31m" << msg << "\033[0m\n";
    }
}
