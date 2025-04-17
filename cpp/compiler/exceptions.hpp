#pragma once
#include <exception>
#include <string>

class compiler_error : public std::exception {
protected:
	std::string message;
public:
	explicit compiler_error(std::string message) : message(std::move(message)) {}
	virtual const char* what() const noexcept = 0;
};

class syntax_error : public compiler_error {
public:
	explicit syntax_error(std::string message) : compiler_error(std::move(message)) {}
	const char* what() const noexcept override {
		return message.c_str();
	}
};

class semantic_error : public compiler_error {
public:
	explicit semantic_error(std::string message) : compiler_error(std::move(message)) {}
	const char* what() const noexcept override {
		return message.c_str();
	}
};
