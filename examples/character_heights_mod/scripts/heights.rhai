// Character Heights: the camera of a player is at the height of its character.
//
// The game gives every character the same height of its head when it stands (1.8) and when it crouches (1.0), whatever its size, so a small character (Robofish) has the camera of a tall one and
// a tall one the camera of a short one.  This sets both from how tall the character is: the top of its model (measured from the game's own models with tools/obdump.py, below) and its scale
// (the table of characters has one: Robofish is 0.65, the others 1.05), against the first character.  A character of a mod, with no height here, has its scale only.  It is done once, when the
// program starts.

fn normal_scale() { 1.05 }     // the scale that the game's heights are for
fn normal_height() { 1.953 }   // the height of the model of the first character, which they are for

// the top of the model of each character, in the pose of its standing animation (the number of the character: the height)
fn model_heights() {
    #{ "0": 1.953, "1": 1.748, "2": 1.907, "3": 1.777, "4": 1.938, "5": 1.726, "6": 1.908, "7": 1.696, "8": 1.875, "9": 1.727, "10": 1.919, "11": 1.748, "12": 1.916, "13": 1.759, "14": 1.939, "15": 1.731, "16": 1.894, "17": 1.754, "18": 1.962, "19": 1.95, "20": 2.007, "21": 1.942, "22": 1.942, "23": 1.727, "24": 1.333, "26": 1.807, "27": 1.951, "28": 1.911, "29": 1.845, "30": 1.939, "31": 1.909, "32": 2.012, "33": 1.914, "34": 1.898, "35": 1.902, "36": 1.956, "37": 1.94, "38": 1.907, "40": 1.981, "41": 1.928, "42": 1.925, "43": 1.919, "44": 1.919, "45": 1.96, "46": 1.986, "47": 1.913, "48": 1.962, "49": 1.947, "50": 1.944, "51": 1.931, "52": 1.909, "53": 1.745, "54": 1.76, "55": 1.944, "56": 1.931, "57": 1.909, "58": 1.745, "60": 1.913, "61": 1.957, "62": 1.937, "63": 1.917, "64": 1.935, "69": 1.877, "70": 1.906, "71": 1.906, "72": 1.906 }
}

fn on_frame() {
    if this.contains("done") { return; }
    this.done = true;
    let heights = model_heights();
    for c in 0..128 {
        let e = chr_entry(c);
        let scale = peekf(e + 40);
        if scale <= 0.0 { continue; }
        let ratio = scale / normal_scale();
        let key = "" + c;
        if heights.contains(key) { ratio *= heights[key] / normal_height(); }
        if ratio < 0.2 || ratio > 3.0 { continue; }
        let floor = peekf(e + 28);
        let stand = peekf(e + 20);
        let crouch = peekf(e + 24);
        pokef(e + 20, floor + (stand - floor) * ratio);
        pokef(e + 24, floor + (crouch - floor) * ratio);
        if env_int("TS_HEIGHTLOG", 0) == 1 && (c == 0 || c == 7 || c == 24 || c == 20) { log("character " + c + ": ratio " + ratio + " head " + peekf(e + 20) + " crouching " + peekf(e + 24)); }
    }
}
