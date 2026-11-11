// Writes web/console/keepawake.mp4: a 2-second, 64x64, all-black, silent H.264 video.
//
// Why: a tablet browser does not let the screen sleep while a video plays. The console
// page plays this clip muted and looping, out of sight, so an iPad showing the console
// stays awake (Safari offers the Screen Wake Lock API only on https pages, and the
// console is served over plain http on the local network).
//
// macOS only (AVFoundation); the output is committed, so other platforms never need it.
//   xcrun swift tools/console_assets/make_keepawake.swift web/console/keepawake.mp4
import AVFoundation
import CoreVideo

let out = URL(fileURLWithPath: CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "keepawake.mp4")
try? FileManager.default.removeItem(at: out)
let writer = try! AVAssetWriter(outputURL: out, fileType: .mp4)
let size = 64
let settings: [String: Any] = [
    AVVideoCodecKey: AVVideoCodecType.h264,
    AVVideoWidthKey: size,
    AVVideoHeightKey: size,
    AVVideoCompressionPropertiesKey: [AVVideoAverageBitRateKey: 8_000, AVVideoMaxKeyFrameIntervalKey: 60],
]
let input = AVAssetWriterInput(mediaType: .video, outputSettings: settings)
let adaptor = AVAssetWriterInputPixelBufferAdaptor(
    assetWriterInput: input,
    sourcePixelBufferAttributes: [kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA,
                                  kCVPixelBufferWidthKey as String: size, kCVPixelBufferHeightKey as String: size])
writer.add(input)
writer.startWriting()
writer.startSession(atSourceTime: .zero)
var buffer: CVPixelBuffer?
CVPixelBufferCreate(nil, size, size, kCVPixelFormatType_32BGRA, nil, &buffer)
CVPixelBufferLockBaseAddress(buffer!, [])
memset(CVPixelBufferGetBaseAddress(buffer!), 0, CVPixelBufferGetDataSize(buffer!)) // black
CVPixelBufferUnlockBaseAddress(buffer!, [])
// 2 s at 15 frames per second.
for frame in 0..<30 {
    while !input.isReadyForMoreMediaData { usleep(1000) }
    adaptor.append(buffer!, withPresentationTime: CMTime(value: CMTimeValue(frame), timescale: 15))
}
input.markAsFinished()
let done = DispatchSemaphore(value: 0)
writer.finishWriting { done.signal() }
done.wait()
print("wrote \(out.path): \(writer.status == .completed ? "ok" : "failed \(String(describing: writer.error))")")
