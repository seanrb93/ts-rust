// Character Heights: the camera of a player is at the height of its character.
//
// The game gives every character the same height of its head when it stands (1.8) and when it crouches (1.0), whatever its size, so a small character (Robofish is 0.65 where the others are
// 1.05) has the camera of a tall one, and a big character of a mod has too low a one.  This sets both as parts of the character's own scale.  It is done once, when the program starts.

fn normal_scale() { 1.05 }   // the scale that the game's heights are for

fn on_frame() {
    if this.contains("done") { return; }
    this.done = true;
    for c in 0..128 {
        let e = chr_entry(c);
        let scale = peekf(e + 40);
        if scale <= 0.0 { continue; }
        let ratio = scale / normal_scale();
        if ratio < 0.3 || ratio > 3.0 { continue; }
        let floor = peekf(e + 28);
        let stand = peekf(e + 20);
        let crouch = peekf(e + 24);
        pokef(e + 20, floor + (stand - floor) * ratio);
        pokef(e + 24, floor + (crouch - floor) * ratio);
        if c == 24 { log("character 24: head " + peekf(e + 20) + " crouching " + peekf(e + 24)); }
    }
}
