#pragma once

#include <ctime>
#include <memory>
#include <string>

#include "nixexpr.hh"

namespace nix {

struct TraceData {
    std::string file;
    const char * type;
    size_t line;

    TraceData() {}
    TraceData(Pos p, const char* t);
    void print(std::ostream & os);
};

template<typename Data, size_t size>
struct TracingChunk {

    struct Entry {
        uint64_t ts_entry;
        uint64_t ts_exit;
        Data data;
    };


    struct EntryRAII {
        Entry* e;

        // Generate a 64bit unsigned integer reprsenting the current
        // time in nanoseconds
        static inline uint64_t now() {
            struct timespec ts;

            clock_gettime(CLOCK_MONOTONIC, &ts);
            const auto ns = uint64_t(ts.tv_nsec);
            const auto s = (uint64_t(ts.tv_sec) * 100000000);
            return s + ns;
        }

        EntryRAII(Entry* e, Data d) : e(e) {
            e->ts_entry = now();
            e->data = d;
        }

        ~EntryRAII() {
            auto n = now();
            e->ts_exit = n;
        }

        Entry* operator->() {
            return e;
        }
    };

    Entry data[size];
    size_t pos;

    TracingChunk(): pos(0) {}


    inline bool has_capacity() const {
        return pos < size - 1;
    }

    inline EntryRAII create(Data d) {
        // assert(has_capacity()); -- we are writing C++ to go fast, who cares about correctness?!?
        auto e = &data[pos++];
        return EntryRAII(e, d);
    }
};

template <typename Data, size_t chunk_size=4096>
struct TracingBuffer {

    typedef TracingChunk<Data, chunk_size> TC;
    // Linked-list of all the chunks that we know about, the last chunk in the list is the latest
    std::list<TC> chunks;
    TC* current_chunk; // FIXME: undefined before alloc_new_chunk

    TracingBuffer() : current_chunk(NULL) {
        alloc_next_chunk();
    }

    inline void alloc_next_chunk() {
        current_chunk = &chunks.emplace_back(TC());
    }

    inline typename TC::EntryRAII create(Data d) {

        if (!current_chunk->has_capacity()) [[unlikely]] {
            alloc_next_chunk();
        }

        return current_chunk->create(d);
    }
};

// FIXME: move this to the header file and the EvalState type so we
// don't use a global state to do tracing.
typedef TracingBuffer<TraceData> TracingBufferT;

// RAII container to ensure that exiting the call actually calls the destructor,
// actual type is std::optional<TracingChunk::EntryRAII>
#define NIX_TRACE(es, pos, type)                                            \
  std::optional<TracingBufferT::TC::EntryRAII> __traceRAII = {};               \
  {                                                                            \
    if ((es).tracingBuffer) [[unlikely]] {                                     \
      __traceRAII =                                                            \
          std::optional((es).tracingBuffer->create(TraceData(pos, type)));  \
    }                                                                          \
  }

// create a "trace point" by passing an eval state reference and an expression
// ptr
#define NIX_TRACE_ES(es, e) NIX_TRACE((es),(es).positions[(e)->getPos()], (e)->showExprType())

// create a top-level trace-point, for usage within the EvalState class. Assumes
// `positions` is in scope.
#define NIX_TRACE_TOP(es,e) NIX_TRACE((es), (es).positions[e->getPos()], "top-level")

}
