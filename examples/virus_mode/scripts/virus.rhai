// Virus, from the later games, as a script of a mod (see docs/scripting.md for what a script can ask and do).
//
// A free for all: everyone is a survivor but one, who starts infected.  An infected character that touches a survivor for a moment infects him.  The last survivor wins.
// Computer characters never shoot: the survivors run from the infected and spread out, the infected go after the survivors on foot.

// the numbers to play with (seconds, and parts of the characters' own speed)
fn grace() { 30.0 }          // nobody can be infected in the first seconds of the match
fn contact_time() { 0.0 }    // how long a touch has to last (0: infected at the first touch)
fn immune() { 6.0 }          // a character just infected cannot infect for this long
fn touch() { 1.3 }           // how close the infected has to be
fn infected_speed() { 0.8 }  // computer characters that are infected run slower than the survivors
fn start_infected() { 1 }    // how many start infected

fn SURV() { 1 }
fn INF() { 2 }

fn hypot(dx, dz) { (dx * dx + dz * dz).sqrt() }

fn on_start() {
    hide_score(true);    // there is no score in this mode
    this.clock = 0.0;
    this.ticks = 0;
    this.wander = [];
    this.next = 1;
    this.order = [];
    this.at = [];
    this.flee = [];
    this.hunt = [];
    this.contact = #{};
    let n = chars();
    let pool = [];
    for i in 0..n {
        this.order.push(-1);
        this.at.push(-1.0);
        this.flee.push([-1, 0.0, 0.0]);
        this.hunt.push([-1, 0.0]);
        this.wander.push([-1, 0.0]);
        pool.push(i);
        set_team(i, SURV());
        tint_clear(i);
        flame(i, false);
        bot_passive(i, !is_player(i));
        bot_speed(i, 1.0);
        bot_pad(i, -1);
        bot_steer_clear(i);
    }
    for z in 0..start_infected() {
        // (chosen by the game's random numbers; the variable is a test aid)
        let forced = env_int("TS_VIRUSZERO", -1);
        let k = rand_int(pool.len());
        if forced >= 0 && z == 0 && pool.contains(forced) {
            for j in 0..pool.len() { if pool[j] == forced { k = j; } }
        }
        let who = pool[k];
        pool.remove(k);
        make_infected(who);
        this.order[who] = 0;
        this.at[who] = 0.0;
        message(who, "You are infected! Infect them all");
        log("character " + who + " starts infected");
    }
}

fn make_infected(i) {
    set_team(i, INF());
    set_colour(i, 4);
    radar_colour(i, 4);    // a green dot on the radar
    tint(i, 0.35, 1.1, 0.35, 0.4);
    flame(i, true);
    bot_speed(i, infected_speed());
    bot_pad(i, -1);
    pain_voice(i);
}

fn infect(b, a) {
    make_infected(b);
    this.order[b] = this.next;
    this.next += 1;
    this.at[b] = this.clock;
    message(b, "You are infected!");
    log("character " + a + " infects character " + b);
}

fn dist(i, j) {
    hypot(x(i) - x(j), z(i) - z(j))
}

fn on_tick() {
    let dt = dt();
    this.clock += dt;
    let n = chars();
    let inf = [];
    let surv = [];
    for i in 0..n {
        let t = team(i);
        if t == INF() {
            inf.push(i);
            // (an infected character groans now and then, in its own voice)
            if alive(i) && rand_int(1500) == 0 { pain_voice(i); }
        } else if t == SURV() {
            surv.push(i);
        }
    }

    // the touch of an infected character infects a survivor once it has lasted a moment
    let touched = #{};
    if this.clock >= grace() * 60.0 {
        for a in inf {
            if !alive(a) { continue; }
            let since = this.at[a];
            if since > 0.0 && this.clock - since < immune() * 60.0 { continue; }
            for b in surv {
                if team(b) != SURV() || !alive(b) { continue; }
                if dist(a, b) < touch() && (y(a) - y(b)).abs() < 1.6 {
                    let key = "" + a + "_" + b;
                    let held = dt;
                    if this.contact.contains(key) { held += this.contact[key]; }
                    if held >= contact_time() * 60.0 {
                        this.infect(b, a);
                    } else {
                        touched[key] = held;
                    }
                }
            }
        }
    }
    this.contact = touched;

    // the game is over when one survivor is left: he wins
    let left = [];
    for i in 0..n { if team(i) == SURV() { left.push(i); } }
    if left.len() <= 1 && this.clock > 60.0 {
        if left.len() == 1 {
            let w = left[0];
            let best = 0;
            for i in 0..n { if score(i) > best { best = score(i); } }
            set_score(w, best + 1);
            message(w, "You are the last survivor!");
            log("character " + w + " is the last survivor");
        } else {
            log("nobody is left uninfected");
        }
        finish();
        return;
    }

    if env_int("TS_TRACE", 0) == 1 {
        this.ticks += 1;
        if this.ticks % 60 == 0 {
            let line = "t" + (this.clock / 60.0).to_int();
            for i in 0..n { line += " [" + i + " t" + team(i) + " " + x(i).to_int() + "," + z(i).to_int() + " st" + bot_state(i) + " pad" + (if team(i) == SURV() { this.flee[i][0] } else { this.hunt[i][0] }) + "]"; }
            log(line);
        }
    }

    // the computer characters
    for i in 0..n {
        if is_player(i) || !alive(i) { continue; }
        if team(i) == SURV() {
            if this.clock < grace() * 60.0 { this.wander_to(i); } else { this.run_away(i, inf, surv); }
        } else {
            if this.clock < grace() * 60.0 { this.wander_to(i); } else { this.run_at(i, surv); }
        }
    }
}

// before anyone can be infected the computer characters walk about from pad to pad
fn wander_to(i) {
    let st = this.wander[i];
    bot_steer_clear(i);
    if st[0] < 0 || this.clock >= st[1] {
        let k = rand_int(pad_count());
        this.wander[i] = [k, this.clock + 420.0];
        bot_pad(i, k);
    }
}

// a survivor goes to a pad that is far from the infected and from the other survivors, and keeps going there until it gets there, or the infected is coming that way
fn run_away(i, inf, surv) {
    let st = this.flee[i];
    let pad = st[0];
    let near = -1;
    let near_d = 1.0e9;
    for a in inf {
        if !alive(a) { continue; }
        let d = dist(i, a);
        if d < near_d { near_d = d; near = a; }
    }
    if near < 0 { bot_pad(i, -1); return; }
    let replan = pad < 0 || this.clock >= st[1];
    if !replan && hypot(pad_x(pad) - x(i), pad_z(pad) - z(i)) < 3.0 { replan = true; }
    if !replan && this.clock >= st[2] && near_d < 10.0 && hypot(pad_x(pad) - x(near), pad_z(pad) - z(near)) < near_d { replan = true; }
    if !replan { return; }

    let best = -1;
    let best_score = -1.0e9;
    for k in 0..pad_count() {
        let px = pad_x(k);
        let pz = pad_z(k);
        let away = 1.0e9;
        for a in inf {
            let d = hypot(px - x(a), pz - z(a));
            if d < away { away = d; }
        }
        let apart = 40.0;
        for b in surv {
            if b == i { continue; }
            let d = hypot(px - x(b), pz - z(b));
            if d < apart { apart = d; }
        }
        let taste = ((i + 1) * 7919 + k * 104729) % 100;
        let s = away - 0.5 * hypot(px - x(i), pz - z(i)) + 0.3 * apart + taste * 0.07;
        if s > best_score { best_score = s; best = k; }
    }
    this.flee[i] = [best, this.clock + 300.0, this.clock + 90.0];
    bot_pad(i, best);
}

// an infected character goes to the pad nearest to the survivor it goes after, and runs straight at it when it is near
fn run_at(i, surv) {
    let st = this.hunt[i];
    if this.clock >= st[1] {
        let target = -1;
        let target_d = 1.0e9;
        for b in surv {
            if !alive(b) { continue; }
            let d = dist(i, b);
            if (y(b) - y(i)).abs() < 2.5 && d < target_d + 0.0 { target_d = d; target = b; }
        }
        if target < 0 {
            for b in surv {
                if !alive(b) { continue; }
                let d = dist(i, b);
                if d < target_d { target_d = d; target = b; }
            }
        }
        let pad = -1;
        if target >= 0 {
            let pad_d = 1.0e9;
            for k in 0..pad_count() {
                let d = hypot(pad_x(k) - x(target), pad_z(k) - z(target));
                if d < pad_d { pad_d = d; pad = k; }
            }
        }
        this.hunt[i] = [target, this.clock + 30.0];
        bot_pad(i, pad);
        st = this.hunt[i];
    }
    let target = st[0];
    if target >= 0 && alive(target) && team(target) == SURV() && dist(i, target) < 15.0 && (y(target) - y(i)).abs() < 2.0 {
        bot_steer(i, x(target), y(target), z(target));
    } else {
        bot_steer_clear(i);
    }
}

// the edge of the screen of an infected player glows green
fn on_hud(player) {
    if team(player) == INF() {
        let alpha = 85 + (30.0 * (this.clock * 0.05).sin()).to_int();
        hud_glow(20, 127, 30, alpha);
    }
}

// the table of results: every character with how long it lasted, the survivor first and then the infected, the last infected first
fn results_per_character() { true }

fn result_key(i) {
    if team(i) == SURV() { 100000 } else { 1000 + (if this.order[i] > 0 { this.order[i] } else { 0 }) }
}

fn result_text(i) {
    let frames = if team(i) == SURV() { this.clock } else { if this.at[i] > 0.0 { this.at[i] } else { 0.0 } };
    let secs = (frames / 60.0).to_int();
    let s = secs % 60;
    let label = if team(i) == SURV() { "Survived " } else { "Infected at " };
    label + (secs / 60) + ":" + (if s < 10 { "0" } else { "" }) + s
}
