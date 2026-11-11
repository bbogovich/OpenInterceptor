// Writes web/console/keepawake.mp4: a 2-second, 96x54, silent H.264 loop of a radar
// sweep: a small "link" indicator shown in the console's corner.
//
// Why: a tablet browser does not let the screen sleep while a video plays. The console
// page plays this clip muted and looping, out of sight, so an iPad showing the console
// stays awake (Safari offers the Screen Wake Lock API only on https pages, and the
// console is served over plain http on the local network). iOS pauses muted videos
// that are not visible on screen, so the clip is shown, small but real, as an indicator.
//
// macOS only (AVFoundation); the output is committed, so other platforms never need it.
//   xcrun swift tools/console_assets/make_keepawake.swift web/console/keepawake.mp4
import AVFoundation
import CoreVideo

let out = URL(fileURLWithPath: CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "keepawake.mp4")
try? FileManager.default.removeItem(at: out)
let writer = try! AVAssetWriter(outputURL: out, fileType: .mp4)
let (w, h) = (96, 54)
let settings: [String: Any] = [
    AVVideoCodecKey: AVVideoCodecType.h264,
    AVVideoWidthKey: w,
    AVVideoHeightKey: h,
    AVVideoCompressionPropertiesKey: [AVVideoAverageBitRateKey: 8_000, AVVideoMaxKeyFrameIntervalKey: 60],
]
let input = AVAssetWriterInput(mediaType: .video, outputSettings: settings)
let adaptor = AVAssetWriterInputPixelBufferAdaptor(
    assetWriterInput: input,
    sourcePixelBufferAttributes: [kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA,
                                  kCVPixelBufferWidthKey as String: w, kCVPixelBufferHeightKey as String: h])
writer.add(input)
writer.startWriting()
writer.startSession(atSourceTime: .zero)
// 2 s at 15 frames per second: a green sweep line turning around the centre, with a
// fading trail, on a dark green background.
for frame in 0..<30 {
    var buffer: CVPixelBuffer?
    CVPixelBufferCreate(nil, w, h, kCVPixelFormatType_32BGRA, nil, &buffer)
    CVPixelBufferLockBaseAddress(buffer!, [])
    let base = CVPixelBufferGetBaseAddress(buffer!)!.assumingMemoryBound(to: UInt8.self)
    let stride = CVPixelBufferGetBytesPerRow(buffer!)
    let sweep = Double(frame) / 30.0 * 2.0 * Double.pi
    for y in 0..<h {
        for x in 0..<w {
            let dx = Double(x) - Double(w) / 2, dy = Double(y) - Double(h) / 2
            let r = (dx * dx + dy * dy).squareRoot()
            var a = sweep - atan2(dy, dx)
            while a < 0 { a += 2 * Double.pi }
            while a >= 2 * Double.pi { a -= 2 * Double.pi }
            let trail = r < 24 ? max(0, 1 - a / 1.2) : 0
            let ring = abs(r - 24) < 0.8 || abs(r - 12) < 0.6 ? 0.35 : 0
            let g = 18 + 200 * max(trail, ring)
            let p = base + y * stride + x * 4
            p[0] = UInt8(10); p[1] = UInt8(min(255, g)); p[2] = UInt8(8 + 40 * trail); p[3] = 255 // BGRA
        }
    }
    CVPixelBufferUnlockBaseAddress(buffer!, [])
    while !input.isReadyForMoreMediaData { usleep(1000) }
    adaptor.append(buffer!, withPresentationTime: CMTime(value: CMTimeValue(frame), timescale: 15))
}
input.markAsFinished()
let done = DispatchSemaphore(value: 0)
writer.finishWriting { done.signal() }
done.wait()
print("wrote \(out.path): \(writer.status == .completed ? "ok" : "failed \(String(describing: writer.error))")")
