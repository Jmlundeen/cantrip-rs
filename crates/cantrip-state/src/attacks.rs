//! Judging a declaration of attackers (CR 508.1a-d): one validator for every restriction, requirement and cost a
//! declaration answers to, from the rules the question holds (`AttackRules`, beside the attackers that may attack, what they
//! may attack and what attacks cost). `AttackRules::judge` says whether a declaration may be made as it is
//! (`AttackVerdict::complete`), why not (`AttackVerdict::issue`), what it costs, and which attacks may be added to it on the
//! way to one that may (`AttackVerdict::addable`). The core checks every answer with it, the scenario harness reads it, and
//! the host gives its verdict to a client as its player declares (`CheckAttacks`), so no client works CR 508.1d out.
//!
//! CR 508.1d counts requirements: the declaration must obey as many as any declaration that obeys every restriction can
//! without paying a cost. A goaded creature (CR 701.15b) is under two for each player who goaded it - attack, and attack a
//! player other than that one - and "attacks that player this combat if able" is one: attacking either of two players may
//! obey as many as the other, and both are legal. A restriction on one attacker at a time (who may attack, and what) leaves
//! the most each attacker's best attack, added up; a cap on how many may attack one player, planeswalker or battle (CR
//! 508.1c) is room the attackers share, which makes the most a maximum-weight assignment with capacities (`search`), solved
//! exactly in polynomial time.

use crate::{AttackCost, ManaCost, ObjectId, PlayerId, Target, count_u32};

/// What a requirement on a declaration of attackers asks of its attacker (CR 508.1d).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttackAim {
    /// That it attack, whatever it attacks: "attacks each combat if able", a goad's first requirement (CR 701.15b).
    Anything,
    /// That it attack this player, planeswalker or battle: "attacks that player this combat if able".
    At(Target),
    /// That it attack a player other than this one: a goad's second requirement, the goader's (CR 701.15b).
    PlayerOtherThan(PlayerId),
}

/// One requirement on a declaration of attackers (CR 508.1d), one for each effect, ability or player it comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequiredAttack {
    pub attacker: ObjectId,
    pub aim: AttackAim,
}

/// The rules of a declaration of attackers past which attacks are allowed (CR 508.1c-d), held in the question so the
/// game-log tree moves through them: the requirements; the most creatures that may attack each player, planeswalker or
/// battle, where something caps it; the most of them any declaration that obeys every restriction obeys without paying a
/// cost, which the declaration must obey too; and one such declaration, the engine's suggestion.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AttackRules {
    pub required: Vec<RequiredAttack>,
    /// The most creatures that may attack each capped player, planeswalker or battle (CR 508.1c), one entry each.
    pub limits: Vec<(Target, u32)>,
    pub most: u32,
    pub suggested: Vec<(ObjectId, Target)>,
}

/// What the rules make of a declaration of attackers.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AttackVerdict {
    /// It may be declared as it is (CR 508.1a-d); its cost, if any, is paid once it is declared (CR 508.1h-j).
    pub complete: bool,
    /// The attacks, by creatures not attacking, that some legal declaration extending it has (empty unless asked for).
    pub addable: Vec<(ObjectId, Target)>,
    /// Why it may not be declared as it is; none when it may.
    pub issue: Option<AttackIssue>,
    /// What declaring it costs (CR 508.1h), zero when nothing.
    pub cost: ManaCost,
    /// Nothing is required of the declaration (the most is 0) and nothing caps how many may attack a target: any
    /// declaration of `addable` attacks, each creature attacking once, is legal, so a client need not ask about each one.
    pub free: bool,
}

/// Why a declaration of attackers may not be made as it is, as a caption reads it: `creature`, then `words`, then `other`
/// where there is one ("Ursine Monstrosity" "must attack" Bot2; "Grizzly Bears" "must attack a player other than" Bot2), with
/// the rule it breaks.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AttackIssue {
    pub creature: ObjectId,
    pub words: String,
    pub other: Option<Target>,
    pub rule: &'static str,
}

impl AttackRules {
    /// Whether `attacker` attacking `at` obeys `r`.
    pub fn obeys(r: &RequiredAttack, attacker: ObjectId, at: Target) -> bool {
        r.attacker == attacker
            && match r.aim {
                AttackAim::Anything => true,
                AttackAim::At(t) => t == at,
                AttackAim::PlayerOtherThan(p) => matches!(at, Target::Player(q) if q != p),
            }
    }

    fn gain(&self, attacker: ObjectId, at: Target) -> u32 {
        count_u32(self.required.iter().filter(|r| Self::obeys(r, attacker, at)).count())
    }

    /// How many creatures may attack `at` (CR 508.1c), or `None` when nothing caps it.
    fn limit(&self, at: Target) -> Option<u32> {
        self.limits.iter().find(|&&(t, _)| t == at).map(|&(_, n)| n)
    }

    /// Whether `at`'s cap leaves room for one more attack among `attacks`.
    fn room(&self, at: Target, attacks: &[(ObjectId, Target)]) -> bool {
        self.limit(at).is_none_or(|n| count_u32(attacks.iter().filter(|x| x.1 == at).count()) < n)
    }

    /// How many of the requirements a declaration obeys (CR 508.1d).
    pub fn obeyed(&self, attacks: &[(ObjectId, Target)]) -> u32 {
        attacks.iter().map(|&(a, t)| self.gain(a, t)).sum()
    }

    /// The best `attacker` can obey: the most any one attack of it among `targets` obeys (and not attacking obeys none),
    /// with one such attack; with `free_only`, among attacks that cost nothing (CR 508.1d). What it asks is what the
    /// requirement demands; whether a cap leaves room for it is the declaration's business (CR 508.1c).
    fn best(&self, attacker: ObjectId, targets: &[Target], costs: &[AttackCost], free_only: bool) -> (u32, Option<Target>) {
        let mut best = (0, None);
        for &t in targets {
            if free_only && !AttackCost::free(costs, attacker, t) {
                continue;
            }
            let g = self.gain(attacker, t);
            if g > best.0 {
                best = (g, Some(t));
            }
        }
        best
    }

    /// Whether a cap applies to any of `targets` (CR 508.1c).
    fn capped(&self, targets: &[Target]) -> bool {
        self.limits.iter().any(|l| targets.contains(&l.0))
    }

    /// The most requirements a declaration of `free` creatures attacking among `targets` obeys on top of `fixed` (CR
    /// 508.1d), with one that does, or `None` when `fixed` itself breaks a cap (CR 508.1c); with `free_only`, attacking only
    /// where it costs nothing. Each creature attacks only once and its requirements count only its own attack, so what one
    /// obeys depends on its attack alone, and a cap is room the creatures share: the most is a maximum-weight assignment of
    /// creatures to targets with the caps as capacities, not each creature's best added up. Each creature first takes its
    /// best attack where nothing caps it; the room at the capped targets then goes where it obeys the most beyond that, by
    /// successive augmenting paths - each the one that adds the most (a creature moved onto a capped target, perhaps
    /// moving the one there on to another), until none adds anything - which is exact (each step keeps the assignment the
    /// best of its size, and what a step adds only falls) and takes time polynomial in the creatures and targets.
    fn search(&self, free: &[ObjectId], targets: &[Target], costs: &[AttackCost], free_only: bool, fixed: &[(ObjectId, Target)]) -> Option<(u32, Vec<(ObjectId, Target)>)> {
        // The capped targets among `targets`, with the room `fixed` leaves at each.
        let mut capped: Vec<(Target, u32)> = Vec::new();
        for &(t, most) in &self.limits {
            let used = count_u32(fixed.iter().filter(|x| x.1 == t).count());
            if used > most {
                return None;
            }
            if targets.contains(&t) {
                capped.push((t, most - used));
            }
        }
        let allowed = |a: ObjectId, t: Target| !free_only || AttackCost::free(costs, a, t);
        // Each creature's best attack where nothing caps it.
        let open: Vec<(u32, Option<Target>)> = free
            .iter()
            .map(|&a| {
                let mut best = (0, None);
                for &t in targets.iter().filter(|&&t| allowed(a, t) && !capped.iter().any(|c| c.0 == t)) {
                    let g = self.gain(a, t);
                    if g > best.0 {
                        best = (g, Some(t));
                    }
                }
                best
            })
            .collect();
        // What attacking each capped target obeys beyond that creature's best open attack, where it is more.
        let more: Vec<Vec<u32>> = free
            .iter()
            .zip(&open)
            .map(|(&a, &(base, _))| {
                capped
                    .iter()
                    .map(|&(t, room)| {
                        let g = if room > 0 && allowed(a, t) { self.gain(a, t) } else { 0 };
                        if g > base { g - base } else { 0 }
                    })
                    .collect()
            })
            .collect();
        let (n, k) = (free.len(), capped.len());
        // The capped target each creature attacks, by index into `capped`, and how many attack each.
        let mut at: Vec<Option<usize>> = vec![None; n];
        let mut load = vec![0u32; k];
        // Each augmenting path takes up one place of room, and there are at most `n` creatures to place.
        for _ in 0..n.min(capped.iter().map(|c| c.1 as usize).sum()) {
            // The most a path adds on reaching each creature (0..n) and each capped target (n..n + k): it starts at a
            // creature attacking no capped target, moves it onto a capped target, which may move the creature attacking
            // there onto another, and so on. A path visits each capped target at most once, so k + 1 rounds settle it.
            let mut reach: Vec<Option<i64>> = (0..n + k).map(|v| (v < n && at[v].is_none()).then_some(0)).collect();
            let mut came = vec![usize::MAX; k];
            for _ in 0..=k + 1 {
                let mut changed = false;
                for i in 0..n {
                    let Some(d) = reach[i] else { continue };
                    for j in 0..k {
                        if more[i][j] > 0 && at[i] != Some(j) {
                            let v = d + i64::from(more[i][j]);
                            if reach[n + j].is_none_or(|w| v > w) {
                                reach[n + j] = Some(v);
                                came[j] = i;
                                changed = true;
                            }
                        }
                    }
                }
                for i in 0..n {
                    let Some(j) = at[i] else { continue };
                    let Some(d) = reach[n + j] else { continue };
                    let v = d - i64::from(more[i][j]);
                    if reach[i].is_none_or(|w| v > w) {
                        reach[i] = Some(v);
                        changed = true;
                    }
                }
                if !changed {
                    break;
                }
            }
            // The path that adds the most, ending where there is room left; none that adds anything, and this is the most.
            let Some(end) = (0..k).filter(|&j| load[j] < capped[j].1 && reach[n + j].is_some_and(|v| v > 0)).max_by_key(|&j| reach[n + j]) else {
                break;
            };
            load[end] += 1;
            let mut j = end;
            for _ in 0..=k {
                let i = came[j];
                let Some(was) = at.get_mut(i) else { break };
                let left = was.replace(j);
                match left {
                    Some(prev) => j = prev,
                    None => break,
                }
            }
        }
        let mut obeyed = self.obeyed(fixed);
        let mut declaration = fixed.to_vec();
        for (i, &a) in free.iter().enumerate() {
            if let Some(j) = at[i] {
                obeyed += open[i].0 + more[i][j];
                declaration.push((a, capped[j].0));
            } else if let (g, Some(t)) = open[i] {
                obeyed += g;
                declaration.push((a, t));
            }
        }
        Some((obeyed, declaration))
    }

    /// The most requirements any declaration of `eligible` attacking among `targets` obeys without paying a cost (CR
    /// 508.1d), with one that does (`search`).
    pub fn solve(&self, eligible: &[ObjectId], targets: &[Target], costs: &[AttackCost]) -> (u32, Vec<(ObjectId, Target)>) {
        self.search(eligible, targets, costs, true, &[]).unwrap_or_default()
    }

    /// What the rules make of `attacks`, a declaration of `eligible` creatures attacking among `targets`, with what attacks
    /// cost (CR 508.1a-d); with `addable`, which attacks may be added on the way to a legal one too.
    pub fn judge(&self, eligible: &[ObjectId], targets: &[Target], costs: &[AttackCost], attacks: &[(ObjectId, Target)], addable: bool) -> AttackVerdict {
        let issue = |creature, words: &str, other, rule| Some(AttackIssue { creature, words: words.to_string(), other, rule });
        let mut found = None;
        let mut mendable = true;
        for (i, &(a, t)) in attacks.iter().enumerate() {
            if attacks[..i].iter().any(|x| x.0 == a) {
                found = issue(a, "can attack only once", None, "508.1a");
            } else if !eligible.contains(&a) || !targets.contains(&t) {
                found = issue(a, "can't attack", Some(t), "508.1c");
            }
            if found.is_some() {
                mendable = false;
                break;
            }
        }
        // CR 508.1c: more creatures declared attacking one target than its cap allows is illegal, though an effect may
        // still put another there once they are declared. Nothing extends a declaration already over a cap, so none is
        // addable.
        if found.is_none() {
            for &(t, n) in &self.limits {
                let mut attacking = attacks.iter().filter(|x| x.1 == t).map(|x| x.0);
                if let Some(creature) = attacking.nth(n as usize) {
                    found = issue(creature, "can't attack", Some(t), "508.1c");
                    mendable = false;
                    break;
                }
            }
        }
        if found.is_none() && self.obeyed(attacks) < self.most {
            // An attacker obeying fewer than it could, and a requirement its best attack obeys that its attack does not.
            found = eligible
                .iter()
                .find_map(|&a| {
                    let now = attacks.iter().find(|x| x.0 == a).map(|x| x.1);
                    let (best, at) = self.best(a, targets, costs, true);
                    let have = now.map_or(0, |t| self.gain(a, t));
                    let at = at?;
                    (have < best).then(|| {
                        let r = self.required.iter().find(|r| Self::obeys(r, a, at) && now.is_none_or(|t| !Self::obeys(r, a, t)))?;
                        match r.aim {
                            AttackAim::Anything => issue(a, "must attack", None, "508.1d"),
                            AttackAim::At(t) => issue(a, "must attack", Some(t), "508.1d"),
                            AttackAim::PlayerOtherThan(p) => issue(a, "must attack a player other than", Some(Target::Player(p)), "508.1d"),
                        }
                    })?
                })
                // Fewer obeyed than the most always leaves some attacker short of its best; should none be found, the
                // declaration is still refused, never complete by accident.
                .or_else(|| issue(attacks.first().map_or(ObjectId::NONE, |x| x.0), "must attack", None, "508.1d"));
        }
        let complete = found.is_none();
        let addable = if addable && mendable && !self.capped(targets) {
            // Every restriction is on one attacker at a time: an attack may be added where the others not yet attacking can
            // make up the rest at their best, paying if need be.
            let declared = self.obeyed(attacks);
            let free: Vec<ObjectId> = eligible.iter().copied().filter(|a| !attacks.iter().any(|x| x.0 == *a)).collect();
            let rest: Vec<(ObjectId, u32)> = free.iter().map(|&a| (a, self.best(a, targets, costs, false).0)).collect();
            let all: u32 = rest.iter().map(|r| r.1).sum();
            let mut list = Vec::new();
            for &(a, best) in &rest {
                for &t in targets {
                    if declared + self.gain(a, t) + all - best >= self.most {
                        list.push((a, t));
                    }
                }
            }
            list
        } else if addable && mendable {
            // A cap is room several creatures share (CR 508.1c), so an attack may be added where that target has room and
            // some declaration extending this one with it still obeys the most (CR 508.1d), paying if need be.
            let free: Vec<ObjectId> = eligible.iter().copied().filter(|a| !attacks.iter().any(|x| x.0 == *a)).collect();
            let mut list = Vec::new();
            for &a in &free {
                let rest: Vec<ObjectId> = free.iter().copied().filter(|&b| b != a).collect();
                for &t in targets {
                    if !self.room(t, attacks) {
                        continue;
                    }
                    let mut fixed = attacks.to_vec();
                    fixed.push((a, t));
                    if self.search(&rest, targets, costs, false, &fixed).is_some_and(|(n, _)| n >= self.most) {
                        list.push((a, t));
                    }
                }
            }
            list
        } else {
            Vec::new()
        };
        // Two attacks of `addable` may still be one too many for a capped target (CR 508.1c), so a capped declaration is
        // never free of asking.
        AttackVerdict { complete, addable, issue: found, cost: AttackCost::total(costs, attacks), free: self.most == 0 && !self.capped(targets) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const U: ObjectId = ObjectId(5);
    const V: ObjectId = ObjectId(6);
    const P2: PlayerId = PlayerId(1);
    const P3: PlayerId = PlayerId(2);
    /// A planeswalker of P2 that one creature at most may attack (Tomik, Orzhov Lawmage).
    const W: Target = Target::Object(ObjectId(7));

    fn capped(at: Target) -> AttackRules {
        AttackRules { limits: vec![(at, 1)], ..Default::default() }
    }

    /// Goaded by P2 and told to attack P2 this combat (three players, Ursine Monstrosity): attacking P2 obeys two of three
    /// (attack, and attack P2), attacking P3 two (attack, and attack a player other than P2); both are legal, and not
    /// attacking is refused.
    #[test]
    fn a_goaded_creature_told_to_attack_its_goader_may_attack_either_player() {
        let targets = [Target::Player(P2), Target::Player(P3)];
        let mut r = AttackRules {
            required: vec![
                RequiredAttack { attacker: U, aim: AttackAim::Anything },
                RequiredAttack { attacker: U, aim: AttackAim::PlayerOtherThan(P2) },
                RequiredAttack { attacker: U, aim: AttackAim::At(Target::Player(P2)) },
            ],
            ..Default::default()
        };
        (r.most, r.suggested) = r.solve(&[U], &targets, &[]);
        assert_eq!(r.most, 2);
        assert!(r.judge(&[U], &targets, &[], &[(U, Target::Player(P2))], false).complete);
        assert!(r.judge(&[U], &targets, &[], &[(U, Target::Player(P3))], false).complete);
        let none = r.judge(&[U], &targets, &[], &[], true);
        assert!(!none.complete && none.addable.len() == 2);
        assert_eq!(none.issue.map(|i| i.rule), Some("508.1d"));
    }

    /// A cost to attack is never paid to obey a requirement (CR 508.1d): with every attack costing something, nothing is
    /// required, and attacking and paying is still legal.
    #[test]
    fn a_requirement_is_never_obeyed_by_paying() {
        let targets = [Target::Player(P2)];
        let costs = [AttackCost { attacker: U, target: Target::Player(P2), cost: ManaCost { generic: 2, ..ManaCost::ZERO }, source: ObjectId(9) }];
        let mut r = AttackRules { required: vec![RequiredAttack { attacker: U, aim: AttackAim::Anything }], ..Default::default() };
        (r.most, r.suggested) = r.solve(&[U], &targets, &costs);
        assert_eq!(r.most, 0);
        assert!(r.judge(&[U], &targets, &costs, &[], false).complete);
        let paid = r.judge(&[U], &targets, &costs, &[(U, Target::Player(P2))], false);
        assert!(paid.complete && paid.cost.generic == 2);
    }

    /// CR 508.1c: a second creature declared attacking a planeswalker no more than one may attack makes the declaration
    /// illegal, and the issue is the caption's.
    #[test]
    fn a_second_attacker_on_a_capped_target_is_illegal() {
        let targets = [Target::Player(P2), W];
        let verdict = capped(W).judge(&[U, V], &targets, &[], &[(U, W), (V, W)], false);
        assert!(!verdict.complete);
        let issue = verdict.issue.expect("an illegal declaration says why");
        assert_eq!((issue.creature, issue.words.as_str(), issue.other, issue.rule), (V, "can't attack", Some(W), "508.1c"));
    }

    /// One creature attacking the capped planeswalker, or another target, is legal.
    #[test]
    fn a_capped_target_allows_one_attacker_and_no_cap_on_another() {
        let targets = [Target::Player(P2), W];
        assert!(capped(W).judge(&[U, V], &targets, &[], &[(U, W)], false).complete);
        assert!(capped(W).judge(&[U, V], &targets, &[], &[(U, W), (V, Target::Player(P2))], false).complete);
        assert!(capped(W).judge(&[U, V], &targets, &[], &[(U, Target::Player(P2)), (V, Target::Player(P2))], false).complete);
    }

    /// Two creatures may attack one target where nothing caps it.
    #[test]
    fn without_a_limit_two_may_attack_one_target() {
        let targets = [Target::Player(P2), W];
        assert!(AttackRules::default().judge(&[U, V], &targets, &[], &[(U, W), (V, W)], false).complete);
    }

    /// Nothing is addable onto a target already at its cap, and what is addable elsewhere still is.
    #[test]
    fn addable_never_adds_a_second_attacker_to_a_capped_target() {
        let targets = [Target::Player(P2), W];
        let verdict = capped(W).judge(&[U, V], &targets, &[], &[(U, W)], true);
        assert!(verdict.complete);
        assert!(verdict.addable.contains(&(V, Target::Player(P2))));
        assert!(!verdict.addable.contains(&(V, W)));
        // Two attacks it offers may still be one too many here, so a client has to ask.
        assert!(!verdict.free);
    }

    /// The most is over declarations, not creatures: with a cap of one on Player B, a creature that must attack Player B
    /// takes the one place there and the creature that may attack anything goes to the planeswalker (CR 508.1d). Filling
    /// Player B with the wrong creature leaves the declaration one requirement short, and is refused.
    #[test]
    fn a_cap_does_not_hide_a_requirement_another_creature_can_obey() {
        let b = Target::Player(P2);
        let targets = [b, W];
        let mut r = capped(b);
        r.required = vec![
            RequiredAttack { attacker: U, aim: AttackAim::Anything },
            RequiredAttack { attacker: V, aim: AttackAim::At(b) },
        ];
        (r.most, r.suggested) = r.solve(&[U, V], &targets, &[]);
        assert_eq!(r.most, 2);
        assert_eq!(r.suggested, vec![(U, W), (V, b)]);
        assert!(r.judge(&[U, V], &targets, &[], &r.suggested, false).complete);
        let wrong = r.judge(&[U, V], &targets, &[], &[(U, b)], false);
        assert!(!wrong.complete);
        assert_eq!(wrong.issue.as_ref().map(|i| (i.creature, i.words.as_str(), i.other, i.rule)), Some((V, "must attack", Some(b), "508.1d")));
    }

    /// Where no cap binds, the most is each creature's best attack added up: two that may attack anything each attack.
    #[test]
    fn a_cap_that_does_not_bind_leaves_the_most_added_up() {
        let b = Target::Player(P2);
        let targets = [b, W];
        let mut r = capped(b);
        r.required = vec![RequiredAttack { attacker: U, aim: AttackAim::Anything }, RequiredAttack { attacker: V, aim: AttackAim::Anything }];
        (r.most, r.suggested) = r.solve(&[U, V], &targets, &[]);
        assert_eq!(r.most, 2);
        assert_eq!(r.suggested.len(), 2);
        assert!(r.suggested.iter().filter(|x| x.1 == b).count() <= 1);
        assert!(r.judge(&[U, V], &targets, &[], &r.suggested, false).complete);
        assert!(r.judge(&[U, V], &targets, &[], &[(U, b), (V, W)], false).complete);
        assert!(!r.judge(&[U, V], &targets, &[], &[(U, b), (V, b)], false).complete);
    }

    /// Two creatures competing for one place at a capped target cannot both take it (CR 508.1c): an attack is addable only
    /// where some declaration extending this one with it still obeys the most (CR 508.1d), so a place at a capped target
    /// that a creature which must attack there needs is not offered to a creature that obeys nothing.
    #[test]
    fn addable_does_not_count_two_creatures_into_one_capped_place() {
        const Y: ObjectId = ObjectId(8);
        const Z: ObjectId = ObjectId(9);
        let b = Target::Player(P2);
        let targets = [b, W];
        let mut r = AttackRules { limits: vec![(b, 2)], ..Default::default() };
        r.required = vec![
            RequiredAttack { attacker: U, aim: AttackAim::Anything },
            RequiredAttack { attacker: V, aim: AttackAim::Anything },
            RequiredAttack { attacker: V, aim: AttackAim::At(b) },
            RequiredAttack { attacker: Y, aim: AttackAim::Anything },
            RequiredAttack { attacker: Y, aim: AttackAim::At(b) },
        ];
        let eligible = [U, V, Y, Z];
        (r.most, r.suggested) = r.solve(&eligible, &targets, &[]);
        assert_eq!(r.most, 5);
        // U holds a place at Player B that only one of V and Y can make up: every declaration extending this one obeys 4.
        let verdict = r.judge(&eligible, &targets, &[], &[(U, b)], true);
        assert!(!verdict.complete);
        assert!(verdict.addable.is_empty());
    }

    /// The suggestion fills a capped target no further than the cap, even where a requirement would send another there.
    #[test]
    fn a_suggestion_never_exceeds_a_limit() {
        let targets = [Target::Player(P2), W];
        let mut r = capped(Target::Player(P2));
        r.required = vec![
            RequiredAttack { attacker: U, aim: AttackAim::Anything },
            RequiredAttack { attacker: U, aim: AttackAim::At(Target::Player(P2)) },
            RequiredAttack { attacker: V, aim: AttackAim::Anything },
            RequiredAttack { attacker: V, aim: AttackAim::At(Target::Player(P2)) },
        ];
        (r.most, r.suggested) = r.solve(&[U, V], &targets, &[]);
        assert_eq!(r.suggested.iter().filter(|x| x.1 == Target::Player(P2)).count(), 1);
        assert_eq!(r.most, 3);
        assert!(r.judge(&[U, V], &targets, &[], &r.suggested, false).complete);
    }

    /// Every declaration of `creatures` among `targets`: each creature attacking one of them or nothing.
    fn every_declaration(creatures: &[ObjectId], targets: &[Target]) -> Vec<Vec<(ObjectId, Target)>> {
        let mut all = vec![Vec::new()];
        for &a in creatures {
            all = all
                .into_iter()
                .flat_map(|d: Vec<(ObjectId, Target)>| {
                    let mut next = vec![d.clone()];
                    for &t in targets {
                        let mut with = d.clone();
                        with.push((a, t));
                        next.push(with);
                    }
                    next
                })
                .collect();
        }
        all
    }

    /// The most, whether a declaration may be made and what may be added to one, against every declaration there is (CR
    /// 508.1c-d): three creatures under every mix of requirements a goad, "attacks each combat if able" and "attacks that
    /// player or planeswalker if able" make, three targets under caps of none, zero, one and two, with and without a cost to
    /// one attack. A declaration within the caps is legal where it obeys as many as any declaration within them that pays
    /// nothing; an attack is addable where some legal declaration extends the declaration with it.
    #[test]
    fn the_most_and_addable_match_every_declaration() {
        const Y: ObjectId = ObjectId(8);
        let creatures = [U, V, Y];
        let targets = [Target::Player(P2), Target::Player(P3), W];
        let menu: [&[AttackAim]; 8] = [
            &[],
            &[AttackAim::Anything],
            &[AttackAim::At(Target::Player(P2))],
            &[AttackAim::At(W)],
            &[AttackAim::Anything, AttackAim::At(W)],
            &[AttackAim::Anything, AttackAim::PlayerOtherThan(P2)],
            &[AttackAim::Anything, AttackAim::PlayerOtherThan(P2), AttackAim::At(Target::Player(P2))],
            &[AttackAim::Anything, AttackAim::Anything],
        ];
        let caps: [&[(Target, u32)]; 7] = [
            &[],
            &[(W, 1)],
            &[(W, 0)],
            &[(Target::Player(P2), 1)],
            &[(Target::Player(P2), 1), (W, 1)],
            &[(Target::Player(P2), 2), (Target::Player(P3), 1), (W, 1)],
            &[(Target::Player(P2), 1), (Target::Player(P3), 0), (W, 2)],
        ];
        let cost = [AttackCost { attacker: U, target: Target::Player(P3), cost: ManaCost { generic: 2, ..ManaCost::ZERO }, source: ObjectId(20) }];
        let declarations = every_declaration(&creatures, &targets);
        let mut partial = vec![Vec::new()];
        partial.extend(creatures.iter().flat_map(|&a| targets.iter().map(move |&t| vec![(a, t)])));
        partial.push(vec![(U, W)]);
        for costs in [&[][..], &cost[..]] {
            for limits in caps {
                for x in 0..menu.len() * menu.len() * menu.len() {
                    let picks = [x % menu.len(), x / menu.len() % menu.len(), x / (menu.len() * menu.len())];
                    let required = creatures
                        .iter()
                        .zip(picks)
                        .flat_map(|(&attacker, p)| menu[p].iter().map(move |&aim| RequiredAttack { attacker, aim }))
                        .collect();
                    let mut r = AttackRules { required, limits: limits.to_vec(), ..Default::default() };
                    let within = |d: &[(ObjectId, Target)]| limits.iter().all(|&(t, n)| count_u32(d.iter().filter(|x| x.1 == t).count()) <= n);
                    let free = |d: &[(ObjectId, Target)]| d.iter().all(|&(a, t)| AttackCost::free(costs, a, t));
                    let most = declarations.iter().filter(|d| within(d) && free(d)).map(|d| r.obeyed(d)).max().unwrap_or(0);
                    (r.most, r.suggested) = r.solve(&creatures, &targets, costs);
                    assert_eq!(r.most, most, "{limits:?} {picks:?}");
                    assert!(within(&r.suggested) && free(&r.suggested) && r.obeyed(&r.suggested) == most, "{limits:?} {picks:?}");
                    for d in &declarations {
                        let legal = within(d) && r.obeyed(d) >= most;
                        assert_eq!(r.judge(&creatures, &targets, costs, d, false).complete, legal, "{limits:?} {picks:?} {d:?}");
                    }
                    for p in partial.iter().filter(|p| within(p)) {
                        let mut want: Vec<(ObjectId, Target)> = Vec::new();
                        for &a in creatures.iter().filter(|&&a| !p.iter().any(|x| x.0 == a)) {
                            for &t in &targets {
                                let extends = declarations.iter().any(|d| {
                                    d.contains(&(a, t)) && p.iter().all(|x| d.contains(x)) && within(d) && r.obeyed(d) >= most
                                });
                                if extends {
                                    want.push((a, t));
                                }
                            }
                        }
                        assert_eq!(r.judge(&creatures, &targets, costs, p, true).addable, want, "{limits:?} {picks:?} {p:?}");
                    }
                }
            }
        }
    }

    /// The assignment takes time polynomial in the creatures and targets: sixty creatures that each attack if able, against
    /// six capped planeswalkers and a capped player, obey as many as there are places.
    #[test]
    fn many_creatures_against_many_caps_are_solved_at_once() {
        let creatures: Vec<ObjectId> = (100..160).map(ObjectId).collect();
        let mut targets = vec![Target::Player(P2)];
        targets.extend((200..206).map(|i| Target::Object(ObjectId(i))));
        let mut r = AttackRules {
            required: creatures.iter().map(|&attacker| RequiredAttack { attacker, aim: AttackAim::Anything }).collect(),
            limits: targets.iter().map(|&t| (t, 1)).collect(),
            ..Default::default()
        };
        (r.most, r.suggested) = r.solve(&creatures, &targets, &[]);
        assert_eq!(r.most, 7);
        assert!(r.judge(&creatures, &targets, &[], &r.suggested, false).complete);
        let verdict = r.judge(&creatures, &targets, &[], &r.suggested[..3], true);
        assert!(!verdict.complete);
        assert_eq!(verdict.addable.len(), (creatures.len() - 3) * 4);
    }
}
