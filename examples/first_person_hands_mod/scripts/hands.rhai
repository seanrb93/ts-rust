// First person hands: the first person guns are held by the arms of the player's own character, as they would be held in real life.
//
// No models are kept for this.  When the game loads a gun, this script says that it wants ob/gun<N>/gun<N>_c<CC>.raw for the first player's character (with _l or _m for a gun held in each hand, the hand on the grip only), and it makes
// that model: it poses the arms of the character on the gun and gives the triangles to the program (`model_add`), which adds them to the gun in the packs, whichever mod that gun comes from.  The folder arms beside this script has
// the arms of each character and how each gun is held, made by examples/first_person_hands_src/make-hands.py.

fn curl_limit() { 150.0 }    // how far round the fingers bend, in degrees
fn wrist_bend() { 0.14 }     // how much of the arm the bend at the wrist takes

// the game asks about the names of the files that start so
fn file_prefixes() { ["ob/gun"] }

// "ob/gun<N>/gun<N>.raw" (the first person model of a gun) gives the one with the arms of the first player's character; any other name gives ""
fn variant_of(name) {
    let character = env_int("TS_PLAYERCHR", player_character(0));   // (the variable is a test aid)
    if character < 0 || character >= 128 { return ""; }
    let rest = name.sub_string(6);
    let slash = rest.index_of("/gun");
    if slash <= 0 { return ""; }
    let digits = rest.sub_string(0, slash);
    for c in digits.to_chars() { if c < '0' || c > '9' { return ""; } }
    if name != "ob/gun" + digits + "/gun" + digits + ".raw" { return ""; }
    let cc = (if character < 10 { "0" } else { "" }) + character;
    "ob/gun" + digits + "/gun" + digits + "_c" + cc + ".raw"
}

// the files to try for a name, first to last (the game uses the plain one when none is there)
fn on_file(name) {
    let v = variant_of(name);
    if v == "" { return []; }
    // (a character with no arms of its own, one a mod added, has the first character's)
    let at = v.index_of("_c");
    [v, v.sub_string(0, at) + "_c00.raw"]
}

// a gun held in both hands: the model that has the hand on the grip only (_m for the left hand's gun, _l for the right's), when there is one
fn on_gun_model(name, left) {
    let v = variant_of(name);
    if v == "" { return ""; }
    v.sub_string(0, v.len() - 4) + (if left { "_m" } else { "_l" }) + ".raw"
}

// ob/gun1/gun1_c54_l.raw: the gun's file, the character and the tag; () when it is not such a name
fn parts_of(name) {
    if !name.ends_with(".raw") { return (); }
    let stem = name.sub_string(0, name.len() - 4);
    let at = stem.index_of("_c");
    if at < 0 { return (); }
    let rest = stem.sub_string(at + 2);
    let under = rest.index_of("_");
    let digits = if under < 0 { rest } else { rest.sub_string(0, under) };
    let tag = if under < 0 { "" } else { rest.sub_string(under) };
    if digits.len() == 0 { return (); }
    for c in digits.to_chars() { if c < '0' || c > '9' { return (); } }
    #{ file: stem.sub_string(0, at) + ".raw", digits: digits, tag: tag }
}

// how each gun is held, read the first time: "<file>|<tag>" -> its hands
fn load_guns() {
    if this.contains("guns") { return; }
    let guns = #{};
    for line in mod_text("scripts/arms/guns.txt").split("\n") {
        let w = line.split(" ");
        if w.len() != 22 { continue; }
        let key = w[0] + "|" + (if w[1] == "-" { "" } else { w[1] });
        let hand = #{
            part: parse_int(w[2]), chain: parse_int(w[3]), mirror: w[4] == "1", scale: parse_float(w[5]), palm: w[6] == "palm",
            fingers: [parse_float(w[7]), parse_float(w[8]), parse_float(w[9])], thumb: [parse_float(w[10]), parse_float(w[11]), parse_float(w[12])], at: [parse_float(w[13]), parse_float(w[14]), parse_float(w[15])],
            width: parse_float(w[16]), limit: parse_float(w[17]), arm: [parse_float(w[18]), parse_float(w[19]), parse_float(w[20])]
        };
        if !guns.contains(key) { guns[key] = []; }
        guns[key].push(hand);
    }
    this.guns = guns;
}

// the arms of a character: its scale, the numbers of its textures, and for each arm its pieces from the hand up: the slots of the textures of the triangles, and three corners for each as x, y, z, s, t, r, g, b
fn read_arms(file) {
    let b = mod_file(file);
    let at = 0;
    let scale = b.f32_at(at); at += 4;
    let count = b.parse_le_int(at, 4); at += 4;
    let numbers = [];
    for i in 0..count { numbers.push(b.parse_le_int(at, 4)); at += 4; }
    let chains = [];
    for chain in 0..2 {
        let pieces = [];
        let piece_count = b.parse_le_int(at, 4); at += 4;
        for p in 0..piece_count {
            let triangles = b.parse_le_int(at, 4); at += 4;
            let slots = [];
            let corners = [];
            for t in 0..triangles {
                slots.push(b.parse_le_int(at, 4)); at += 4;
                for c in 0..3 {
                    for k in 0..5 { corners.push(b.f32_at(at + 4 * k)); }
                    corners.push(b[at + 20]); corners.push(b[at + 21]); corners.push(b[at + 22]);
                    at += 24;
                }
            }
            pieces.push(#{ slots: slots, corners: corners });
        }
        chains.push(pieces);
    }
    #{ scale: scale, numbers: numbers, chains: chains }
}

fn has_model(name) {
    let parts = parts_of(name);
    if parts == () { return false; }
    this.load_guns();
    this.guns.contains(parts.file + "|" + parts.tag) && mod_has_file("scripts/arms/c" + parts.digits + ".arm")
}

// (for the test aid TS_MODELDUMP)
fn model_names() {
    this.load_guns();
    let out = [];
    for c in 0..128 {
        let digits = (if c < 10 { "0" } else { "" }) + c;
        if !mod_has_file("scripts/arms/c" + digits + ".arm") { continue; }
        for key in this.guns.keys() {
            let at = key.index_of("|");
            let file = key.sub_string(0, at);
            out.push(file.sub_string(0, file.len() - 4) + "_c" + digits + key.sub_string(at + 1) + ".raw");
        }
    }
    out
}

// vectors
fn v_add(a, b) { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn v_sub(a, b) { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn v_times(a, s) { [a[0] * s, a[1] * s, a[2] * s] }
fn v_dot(a, b) { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn v_cross(a, b) { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn v_size(a) { sqrt(v_dot(a, a)) }
fn v_unit(a) { v_times(a, 1.0 / v_size(a)) }
// a turn is its three rows
fn t_apply(m, v) { [v_dot(m[0], v), v_dot(m[1], v), v_dot(m[2], v)] }
fn t_multiply(a, b) {
    let c = [[b[0][0], b[1][0], b[2][0]], [b[0][1], b[1][1], b[2][1]], [b[0][2], b[1][2], b[2][2]]];
    [[v_dot(a[0], c[0]), v_dot(a[0], c[1]), v_dot(a[0], c[2])], [v_dot(a[1], c[0]), v_dot(a[1], c[1]), v_dot(a[1], c[2])], [v_dot(a[2], c[0]), v_dot(a[2], c[1]), v_dot(a[2], c[2])]]
}
// the turn with the unit: 1 + s k + c k k
fn t_from(k, s, c) {
    let kk = t_multiply(k, k);
    let out = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    for r in 0..3 { for q in 0..3 { out[r][q] = (if r == q { 1.0 } else { 0.0 }) + s * k[r][q] + c * kk[r][q]; } }
    out
}
// the turn that takes the direction a to the direction b
fn t_between(a, b) {
    let a = v_unit(a);
    let b = v_unit(b);
    let v = v_cross(a, b);
    let c = v_dot(a, b);
    if c > 0.9999999 { return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]; }
    if c < -0.9999999 { return [[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, -1.0]]; }
    t_from([[0.0, -v[2], v[1]], [v[2], 0.0, -v[0]], [-v[1], v[0], 0.0]], 1.0, 1.0 / (1.0 + c))
}

fn points_of(piece) {
    let out = [];
    let c = piece.corners;
    let i = 0;
    while i < c.len() {
        out.push([c[i], c[i + 1], c[i + 2]]);
        i += 8;
    }
    out
}

fn mean_of(points) {
    let total = [0.0, 0.0, 0.0];
    for p in points { total = v_add(total, p); }
    v_times(total, 1.0 / points.len().to_float())
}

// how the points of an arm go on a gun: the hand is turned to the gun with its fingers bent round what they hold, and the rest of the arm comes from where the elbow is, with a bend at the wrist
fn pose_of(hand, pieces, character_scale) {
    let scale = hand.scale * character_scale;
    let whole = pieces.len() == 1;
    let first = points_of(pieces[0]);
    let grip = first;
    let elbow = [];
    if whole {
        // (an arm in one piece: the hand is its lowest sixth, the elbow its highest)
        let by_y = first;
        by_y.sort(|a, b| if a[1] < b[1] { -1 } else if a[1] > b[1] { 1 } else { 0 });
        let k = by_y.len() / 6;
        if k < 1 { k = 1; }
        grip = by_y.extract(0, k);
        elbow = by_y.extract(by_y.len() - k, k);
    } else {
        elbow = points_of(pieces[pieces.len() - 1]);
    }
    let middle = mean_of(grip);
    let far = mean_of(elbow);
    let down = v_unit(v_sub(middle, far));
    let thumb_rest = v_unit(v_sub([0.0, 0.0, 1.0], v_times(down, down[2])));
    let palm_rest = v_times(v_cross(down, thumb_rest), if middle[0] > 0.0 { 1.0 } else { -1.0 });
    let wrist_u = 1.0e30;
    let tip_u = -1.0e30;
    let along = [];
    for p in grip {
        let u = v_dot(v_sub(p, middle), down);
        along.push(u);
        if u < wrist_u { wrist_u = u; }
        if u > tip_u { tip_u = u; }
    }
    let wrist = v_add(middle, v_times(down, wrist_u));
    let knuckle_u = wrist_u + 0.55 * (tip_u - wrist_u);
    let palm_face = -1.0e30;
    let thumb_most = -1.0e30;
    for i in 0..grip.len() {
        let d = v_sub(grip[i], middle);
        let t = v_dot(d, thumb_rest);
        if t > thumb_most { thumb_most = t; }
        if along[i] < knuckle_u {
            let n = v_dot(d, palm_rest);
            if n > palm_face { palm_face = n; }
        }
    }
    let fingers = v_unit(hand.fingers);
    let thumb = v_unit(v_sub(hand.thumb, v_times(fingers, v_dot(hand.thumb, fingers))));
    // (a turn takes down, the thumb and the palm of the rest to the fingers, the thumb and what follows of them)
    let rest = [down, thumb_rest, v_cross(down, thumb_rest)];
    let goal = [fingers, thumb, v_cross(fingers, thumb)];
    let turn = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    for r in 0..3 { for q in 0..3 { for k in 0..3 { turn[r][q] += goal[k][r] * rest[k][q]; } } }
    let radius = hand.width / 2.0;
    if radius > 0.05 { radius = 0.05; }
    if radius < 0.018 { radius = 0.018; }
    radius /= if scale > 1.0e-6 { scale } else { 1.0e-6 };
    let hand_at = if hand.palm {
        v_sub(hand.at, v_times(t_apply(turn, v_times(palm_rest, palm_face)), scale))
    } else {
        let knuckle_rest = v_add(v_add(middle, v_times(down, knuckle_u)), v_times(palm_rest, palm_face));
        v_sub(hand.at, v_times(t_apply(turn, v_sub(knuckle_rest, middle)), scale))
    };
    let arm_turn = t_between(v_sub(far, wrist), hand.arm);
    // (the arm is rolled about itself so that its thumb side is the hand's: no twist at the wrist)
    let axis = v_unit(hand.arm);
    let got = t_apply(arm_turn, thumb_rest);
    got = v_sub(got, v_times(axis, v_dot(got, axis)));
    let want = v_sub(thumb, v_times(axis, v_dot(thumb, axis)));
    if v_size(got) > 1.0e-6 && v_size(want) > 1.0e-6 {
        let angle = atan(v_dot(v_cross(got, want), axis), v_dot(got, want));
        arm_turn = t_multiply(t_from([[0.0, -axis[2], axis[1]], [axis[2], 0.0, -axis[0]], [-axis[1], axis[0], 0.0]], sin(angle), 1.0 - cos(angle)), arm_turn);
    }
    #{
        whole: whole, mirror: hand.mirror, scale: scale, limit: hand.limit * 3.141592653589793 / 180.0, radius: radius, middle: middle, down: down, palm_rest: palm_rest, thumb_rest: thumb_rest,
        knuckle_u: knuckle_u, tip_u: tip_u, palm_face: palm_face, thumb_most: thumb_most, wrist: wrist, arm_rest: v_unit(v_sub(far, wrist)), turn: turn, arm_turn: arm_turn, hand_at: hand_at,
        wrist_at: v_add(hand_at, v_times(t_apply(turn, v_sub(wrist, middle)), scale))
    }
}

// the points (x, y, z, one after another) on the gun
fn place_all(pose, points, of_hand, every) {
    let whole = pose.whole;
    let scale = pose.scale;
    let mx = pose.middle[0]; let my = pose.middle[1]; let mz = pose.middle[2];
    let dx = pose.down[0]; let dy = pose.down[1]; let dz = pose.down[2];
    let px = pose.palm_rest[0]; let py = pose.palm_rest[1]; let pz = pose.palm_rest[2];
    let tx = pose.thumb_rest[0]; let ty = pose.thumb_rest[1]; let tz = pose.thumb_rest[2];
    let wx = pose.wrist[0]; let wy = pose.wrist[1]; let wz = pose.wrist[2];
    let rx = pose.arm_rest[0]; let ry = pose.arm_rest[1]; let rz = pose.arm_rest[2];
    let knuckle_u = pose.knuckle_u; let spread = pose.tip_u - knuckle_u;
    if spread < 1.0e-6 { spread = 1.0e-6; }
    let round = pose.palm_face + pose.radius;
    let thumb_most = if pose.thumb_most > 1.0e-6 { pose.thumb_most } else { 1.0e-6 };
    let limit = pose.limit;
    let t0 = pose.turn[0]; let t1 = pose.turn[1]; let t2 = pose.turn[2];
    let a0 = pose.arm_turn[0]; let a1 = pose.arm_turn[1]; let a2 = pose.arm_turn[2];
    let t00 = t0[0]; let t01 = t0[1]; let t02 = t0[2]; let t10 = t1[0]; let t11 = t1[1]; let t12 = t1[2]; let t20 = t2[0]; let t21 = t2[1]; let t22 = t2[2];
    let a00 = a0[0]; let a01 = a0[1]; let a02 = a0[2]; let a10 = a1[0]; let a11 = a1[1]; let a12 = a1[2]; let a20 = a2[0]; let a21 = a2[1]; let a22 = a2[2];
    let hx = pose.hand_at[0]; let hy = pose.hand_at[1]; let hz = pose.hand_at[2];
    let ax = pose.wrist_at[0]; let ay = pose.wrist_at[1]; let az = pose.wrist_at[2];
    let bend_over = wrist_bend();
    let flip = if pose.mirror { -1.0 } else { 1.0 };
    let out = [];
    let i = 0;
    let end = points.len();
    while i < end {
        let x = points[i] - mx; let y = points[i + 1] - my; let z = points[i + 2] - mz;
        i += every;
        let ox = 0.0; let oy = 0.0; let oz = 0.0;
        if whole {
            ox = hx + (a00 * x + a01 * y + a02 * z) * scale;
            oy = hy + (a10 * x + a11 * y + a12 * z) * scale;
            oz = hz + (a20 * x + a21 * y + a22 * z) * scale;
        } else if of_hand {
            let u = x * dx + y * dy + z * dz;
            let n = x * px + y * py + z * pz;
            let t = x * tx + y * ty + z * tz;
            if u > knuckle_u {
                // (the fingers fold: each is bent round what is held, the whole of its length on the arc, so that no tip stands out past it)
                let side = (t / thumb_most - 0.35) / 0.4;
                if side < 0.0 { side = 0.0; } else if side > 1.0 { side = 1.0; }
                let fade = 1.0 - side * side * (3.0 - 2.0 * side);
                let angle = (u - knuckle_u) / spread * limit * fade;
                let reach = round - n;
                u = knuckle_u + (u - knuckle_u) * (1.0 - fade) + reach * sin(angle);
                n = round - reach * cos(angle);
            }
            let qx = dx * u + px * n + tx * t; let qy = dy * u + py * n + ty * t; let qz = dz * u + pz * n + tz * t;
            ox = hx + (t00 * qx + t01 * qy + t02 * qz) * scale;
            oy = hy + (t10 * qx + t11 * qy + t12 * qz) * scale;
            oz = hz + (t20 * qx + t21 * qy + t22 * qz) * scale;
        } else {
            let fx = points[i - every] - wx; let fy = points[i - every + 1] - wy; let fz = points[i - every + 2] - wz;
            // (the arm bends over its first part: a stiff arm would stick out at the top of the wrist)
            let bend = (fx * rx + fy * ry + fz * rz) / bend_over;
            if bend < 0.0 { bend = 0.0; } else if bend > 1.0 { bend = 1.0; }
            bend = bend * bend * (3.0 - 2.0 * bend);
            let keep = 1.0 - bend;
            ox = bend * (ax + (a00 * fx + a01 * fy + a02 * fz) * scale) + keep * (hx + (t00 * x + t01 * y + t02 * z) * scale);
            oy = bend * (ay + (a10 * fx + a11 * fy + a12 * fz) * scale) + keep * (hy + (t10 * x + t11 * y + t12 * z) * scale);
            oz = bend * (az + (a20 * fx + a21 * fy + a22 * fz) * scale) + keep * (hz + (t20 * x + t21 * y + t22 * z) * scale);
        }
        out.push(ox * flip); out.push(oy); out.push(oz);
    }
    out
}

// the program asks for the model: the arms are given to it, and it is told which gun they go on
fn make_model(name) {
    let parts = parts_of(name);
    if parts == () { return ""; }
    this.load_guns();
    let key = parts.file + "|" + parts.tag;
    if !this.guns.contains(key) { return ""; }
    let file = "scripts/arms/c" + parts.digits + ".arm";
    if !mod_has_file(file) { return ""; }
    if !this.contains("arms") { this.arms = #{}; }
    if !this.arms.contains(parts.digits) { this.arms[parts.digits] = read_arms(file); }
    let character = this.arms[parts.digits];
    for hand in this.guns[key] {
        let pieces = character.chains[hand.chain];
        if pieces.len() == 0 { continue; }
        let pose = pose_of(hand, pieces, character.scale);
        let values = [];
        for index in 0..pieces.len() {
            let piece = pieces[index];
            let of_hand = index == 0;
            let corners = piece.corners;
            let placed = place_all(pose, corners, of_hand, 8);
            let centre = place_all(pose, mean_of(points_of(piece)), of_hand, 3);
            let triangles = piece.slots.len();
            for t in 0..triangles {
                values.push(character.numbers[piece.slots[t]]);
                values.push(centre[0]); values.push(centre[1]); values.push(centre[2]);
                for k in 0..3 {
                    let c = t * 3 + (if hand.mirror { 2 - k } else { k });
                    values.push(placed[c * 3]); values.push(placed[c * 3 + 1]); values.push(placed[c * 3 + 2]);
                    let at = c * 8 + 3;
                    values.push(corners[at]); values.push(corners[at + 1]); values.push(corners[at + 2]); values.push(corners[at + 3]); values.push(corners[at + 4]);
                }
            }
        }
        model_add(hand.part, values);
    }
    parts.file
}
