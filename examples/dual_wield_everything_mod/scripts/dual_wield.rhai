// Every Weapon (x2): every weapon has a dual (x2) version in the weapon sets, and an unarmed player has a left fist as well as a right.
//
// The program has the dual versions in its table of weapons (the game's own for eight of them, one added for each of the others) and a mirrored left fist; this script says when they are used.
// A mod could list only some of the pairs: `lists_pair` is asked for each weapon (a number of the game's table of weapon combinations).

// the weapon page lists the pair of the weapon `single`
fn lists_pair(single) { true }

// a set of weapons may have the pair `pair` of the weapon `single` (when not, it has the single weapon)
fn allows_pair(pair, single) { true }

// an unarmed player punches with the left fist and the right in turn
fn alternate_fists() { setting("both_fists", true) }
