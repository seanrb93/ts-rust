// Bots Punch: bots that have no weapon (Always start with gun: No, and none to pick up) go after their target and punch it, instead of standing still.

fn reach() { 2.5 }       // how near a bot has to be to hit
fn height() { 1.5 }      // and how near in height (the same floor)
fn damage_done() { 0.5 } // of a character's health (a pistol shot is 0.3)
fn wait() { 45.0 }       // frames of 60 to the second between two blows of a bot

fn on_start() {
    this.cool = [];
    for i in 0..chars() { this.cool.push(0.0); }
}

// the game looked for a weapon for a bot and found none to pick up: an unarmed bot chooses its target instead
fn on_bot_idle(i) {
    if bot_unarmed(i) {
        bot_set_state(i, 128);
        return true;
    }
    false
}

fn on_tick() {
    let dt = dt();
    for i in players()..chars() {
        if i >= this.cool.len() { continue; }
        if this.cool[i] > 0.0 { this.cool[i] -= dt; }
        if !alive(i) || !bot_unarmed(i) { continue; }
        let t = bot_target(i);
        if t < 0 || !alive(t) { continue; }
        let dx = x(t) - x(i);
        let dz = z(t) - z(i);
        if dx * dx + dz * dz <= reach() * reach() && (y(t) - y(i)).abs() <= height() && this.cool[i] <= 0.0 {
            this.cool[i] = wait();
            damage(t, i, damage_done());
            log("bot " + i + " punches " + t);
            sound(171 + rand_int(2), i);
        }
    }
}
