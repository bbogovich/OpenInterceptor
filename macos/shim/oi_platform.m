// macOS platform shim for OpenInterceptor.
//
// Compiled by build.rs only when targeting macOS (xcrun clang + ar, no extra crates) and
// called from src/platform/macos.rs. It exposes one tiny C function that reads a game
// controller through GameController.framework (Xbox, PlayStation, Switch Pro and MFi
// pads with an extended-gamepad profile) and reports it in XInput's layout.
//
// Adapted from the Looking Glass macOS port's lg_platform.m (MIT licence).

#import <GameController/GameController.h>
#include <stdint.h>

// Must match `OiPad` in src/platform/macos.rs and `PadState` in src/input/gamepad.rs.
typedef struct {
    uint16_t buttons;
    uint8_t lt;
    uint8_t rt;
    int16_t lx;
    int16_t ly;
    int16_t rx;
    int16_t ry;
} OiPad;

// XInput button bits (see Button::bit in src/input/gamepad.rs).
enum {
    OI_DPAD_UP = 0x0001, OI_DPAD_DOWN = 0x0002, OI_DPAD_LEFT = 0x0004, OI_DPAD_RIGHT = 0x0008,
    OI_START = 0x0010, OI_BACK = 0x0020, OI_LEFT_THUMB = 0x0040, OI_RIGHT_THUMB = 0x0080,
    OI_LEFT_SHOULDER = 0x0100, OI_RIGHT_SHOULDER = 0x0200,
    OI_A = 0x1000, OI_B = 0x2000, OI_X = 0x4000, OI_Y = 0x8000,
};

// A stick value (-1...1) as an XInput thumb value.
static int16_t oi_axis(float value) {
    if (value != value) return 0;  // NaN
    if (value > 1.0f) value = 1.0f;
    if (value < -1.0f) value = -1.0f;
    return (int16_t)(value * 32767.0f);
}

// A trigger value (0...1) as an XInput trigger value.
static uint8_t oi_trigger(float value) {
    if (value != value || value <= 0.0f) return 0;
    if (value >= 1.0f) return 255;
    return (uint8_t)(value * 255.0f + 0.5f);
}

// `bit` when the button is held, so buttons can be OR-ed into the mask.
static uint16_t oi_bit(GCControllerButtonInput *button, uint16_t bit) {
    return (button != nil && button.isPressed) ? bit : 0;
}

// The `index`-th connected controller with an extended gamepad profile, or nil.
static GCExtendedGamepad *oi_gamepad(uint32_t index) {
    uint32_t seen = 0;
    for (GCController *controller in [GCController controllers]) {
        GCExtendedGamepad *pad = controller.extendedGamepad;
        if (pad == nil) continue;
        if (seen == index) return pad;
        seen++;
    }
    return nil;
}

// Returns 1 and fills `out` when controller `index` is connected, otherwise 0.
int32_t oi_pad_state(uint32_t index, OiPad *out) {
    if (out == NULL) return 0;
    @autoreleasepool {
        GCExtendedGamepad *pad = oi_gamepad(index);
        if (pad == nil) return 0;
        OiPad s = {0};
        s.buttons = oi_bit(pad.dpad.up, OI_DPAD_UP) | oi_bit(pad.dpad.down, OI_DPAD_DOWN) |
                    oi_bit(pad.dpad.left, OI_DPAD_LEFT) | oi_bit(pad.dpad.right, OI_DPAD_RIGHT) |
                    oi_bit(pad.buttonMenu, OI_START) | oi_bit(pad.buttonOptions, OI_BACK) |
                    oi_bit(pad.leftThumbstickButton, OI_LEFT_THUMB) |
                    oi_bit(pad.rightThumbstickButton, OI_RIGHT_THUMB) |
                    oi_bit(pad.leftShoulder, OI_LEFT_SHOULDER) |
                    oi_bit(pad.rightShoulder, OI_RIGHT_SHOULDER) | oi_bit(pad.buttonA, OI_A) |
                    oi_bit(pad.buttonB, OI_B) | oi_bit(pad.buttonX, OI_X) | oi_bit(pad.buttonY, OI_Y);
        s.lt = oi_trigger(pad.leftTrigger.value);
        s.rt = oi_trigger(pad.rightTrigger.value);
        // GameController and XInput both report stick Y as positive when pushed up.
        s.lx = oi_axis(pad.leftThumbstick.xAxis.value);
        s.ly = oi_axis(pad.leftThumbstick.yAxis.value);
        s.rx = oi_axis(pad.rightThumbstick.xAxis.value);
        s.ry = oi_axis(pad.rightThumbstick.yAxis.value);
        *out = s;
        return 1;
    }
}
