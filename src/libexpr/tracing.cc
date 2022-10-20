#include "tracing.hh"

static uint64_t trace_counter = 0;

namespace nix {
    TraceData::TraceData(Pos p, const char* t):
        // If the origin is a String don't use the location as
        // otherwise we emit the entire input string as "file".
        file((p.origin == foString) ? "<string>" :
             ((p.origin == foStdin) ? "<stdin>" : p.file)),
        type(t),
        line((p.origin == foString || p.origin == foStdin) ? 0 : p.line),
        id(trace_counter++),
        invalid(false)
    {}

    void TraceData::print(std::ostream & os) {
        os << id << " " << (file.size() > 0 ? file : "<undefined>") << " " << (type ? type : "n/a") << " " << line << (invalid ? " invalid" : "");
    }
}
