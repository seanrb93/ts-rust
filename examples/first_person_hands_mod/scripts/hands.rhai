// First person hands: the first person guns are held by the arms of the player's own character.
//
// The program makes the models when they are loaded (OB.PAK/ob/gun<N>/gun<N>_c<CC>.raw, for each character, and _l and _m for a gun that is held in both hands, with the hand on the grip only), from the mod's
// folder arms and the gun in the packs.  This script says which the game loads: the one of the first player's character, and the plain model when there is none.

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

// the files to try for a name, first to last (the game uses the plain one when none is in the packs)
fn on_file(name) {
    let v = variant_of(name);
    if v == "" { return []; }
    // (a character with no arms of its own, one a mod added, has the first character's)
    let slash = v.index_of("_c");
    [v, v.sub_string(0, slash) + "_c00.raw"]
}

// a gun held in both hands: the model that has the hand on the grip only (_m for the left hand's gun, _l for the right's), when the mod has one
fn on_gun_model(name, left) {
    let v = variant_of(name);
    if v == "" { return ""; }
    v.sub_string(0, v.len() - 4) + (if left { "_m" } else { "_l" }) + ".raw"
}
