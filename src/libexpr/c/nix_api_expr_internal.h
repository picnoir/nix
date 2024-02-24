#ifndef NIX_API_EXPR_INTERNAL_H
#define NIX_API_EXPR_INTERNAL_H

#include "eval.hh"
#include "attr-set.hh"

struct EvalState
{
    nix::EvalState state;
};

struct BindingsBuilder
{
    nix::BindingsBuilder builder;
};

struct nix_string_return
{
    std::string str;
};

struct nix_printer
{
    std::ostream & s;
};

struct nix_string_context
{
    nix::NixStringContext & ctx;
};

#endif // NIX_API_EXPR_INTERNAL_H
