#pragma once

#include "lexer.hpp"
#include "parser.hpp"
#include <iostream>

void compile(std::string& source, std::ostream& os);
