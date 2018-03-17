use stdweb::unstable::TryInto;

pub fn gamepad_init() {
    js! {
        window.pads=[];
        if (navigator.getGamepads === undefined) {
            console.log("warning : no gamepad support on this browser");
        } else {
            window.addEventListener("gamepadconnected", function(e) {
                if (e.gamepad) {
                    console.log("gamepad["+e.gamepad.index+"] id "+e.gamepad.id+" connected.");
                    window.pads[e.gamepad.index] = e.gamepad;
                }
            });
            window.addEventListener("gamepaddisconnected", function(e) {
                if (e.gamepad) {
                    console.log("gamepad["+e.gamepad.index+"] id "+e.gamepad.id+" disconnected.");
                    window.pads[e.gamepad.index] = undefined;
                }
            });
        }
    };
}

pub fn gamepad_axis(player_num: i32) -> (f32, f32) {
    let x: f64 = js! {
        if (navigator.userAgent.toLowerCase().indexOf("chrome") != -1) {
            var gp = navigator.getGamepads();
            for (var i=0; i < gp.length; i++) {
                if (gp[i]!=null) {
                    window.pads[gp[i].index]=gp[i];
                }
            }
        }
        if ( window.pads[@{player_num}] ) {
            return window.pads[@{player_num}].axes[0];
        } else {
            return 0.0;
        }
    }.try_into()
        .unwrap();
    let y: f64 = js! {
        if ( window.pads[@{player_num}] ) {
            return window.pads[@{player_num}].axes[1];
        } else {
            return 0.0;
        }
    }.try_into()
        .unwrap();
    (x as f32, y as f32)
}
pub fn gamepad_button(player_num: i32, button_num: i32) -> bool {
    let ret = js! {
        if (navigator.userAgent.toLowerCase().indexOf("chrome") != -1) {
            var gp = navigator.getGamepads();
            for (var i=0; i < gp.length; i++) {
                if (gp[i]!=null) {
                    window.pads[gp[i].index]=gp[i];
                }
            }
        }
        if ( window.pads[@{player_num}] ) {
            var button = window.pads[@{player_num}].buttons[@{button_num}];
            if (typeof button == "object") {
                return button.pressed;
            } else {
                return button == 1.0;
            }
        } else {
            return false;
        }
    }.try_into()
        .unwrap();
    ret
}
