#include "tracing.hh"

namespace nix {
    TraceData::TraceData(Pos p, const char* t):
        // If the origin is a String don't use the location as
        // otherwise we emit the entire input string as "file".
        file((p.origin == foString) ? "<string>" :
             ((p.origin == foStdin) ? "<stdin>" : p.file)),
        type(t),
        line((p.origin == foString || p.origin == foStdin) ? 0 : p.line)
    {}

    void TraceData::print(std::ostream & os) {
        os << ((size_t)this) << " " << (file.size() > 0 ? file : "<undefined>") << " " << type << " " << line;
    }
}
