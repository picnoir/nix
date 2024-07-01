#include <iostream>
#include "tracing-file-system-object-sink.hh"

namespace nix::test {

void TracingFileSystemObjectSink::createDirectory(const Path & path)
{
    std::cerr << "createDirectory(" << path << ")\n";
    sink.createDirectory(path);
}

void TracingFileSystemObjectSink::createRegularFile(const Path & path, std::function<void(CreateRegularFileSink &)> fn)
{
    std::cerr << "createRegularFile(" << path << ")\n";
    sink.createRegularFile(path, [&](CreateRegularFileSink & crf) {
        // We could wrap this and trace about the chunks of data and such
        fn(crf);
    });
}

void TracingFileSystemObjectSink::createSymlink(const Path & path, const std::string & target)
{
    std::cerr << "createSymlink(" << path << ", target: " << target << ")\n";
    sink.createSymlink(path, target);
}

void TracingExtendedFileSystemObjectSink::createHardlink(const Path & path, const CanonPath & target)
{
    std::cerr << "createHardlink(" << path << ", target: " << target << ")\n";
    sink.createHardlink(path, target);
}

} // namespace nix::test
