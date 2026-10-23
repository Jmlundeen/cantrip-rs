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
//! the most each attacker's best attack, added up; a restriction on the declaration as a whole - a cap on how many may attack
//! one player, planeswalker or battle (CR 508.1c) - makes it a search over declarations, as blocks are, which `solve` is.

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

    /// The attacks of `attacker` among `targets` that obey something, the most first: an attack obeying none is no part of a
    /// declaration that obeys the most; with `free_only`, among attacks that cost nothing (CR 508.1d).
    fn options(&self, attacker: ObjectId, targets: &[Target], costs: &[AttackCost], free_only: bool) -> Vec<(Target, u32)> {
        let mut options: Vec<(Target, u32)> = targets
            .iter()
            .copied()
            .filter(|&t| !free_only || AttackCost::free(costs, attacker, t))
            .map(|t| (t, self.gain(attacker, t)))
            .filter(|&(_, g)| g > 0)
            .collect();
        options.sort_by_key(|&(_, g)| std::cmp::Reverse(g));
        options
    }

    /// The most requirements a declaration of `free` creatures attacking among `targets` obeys on top of `fixed` (CR
    /// 508.1d), with one that does, or `None` when `fixed` itself breaks a cap (CR 508.1c). A cap is a budget several
    /// creatures share, so this is a search over declarations - each creature its best attack first, cut by what the
    /// creatures still to come could obey - rather than each creature's best added up; it stops once `enough` is obeyed.
    fn search(
        &self,
        free: &[ObjectId],
        targets: &[Target],
        costs: &[AttackCost],
        free_only: bool,
        fixed: &[(ObjectId, Target)],
        enough: Option<u32>,
    ) -> Option<(u32, Vec<(ObjectId, Target)>)> {
        let options: Vec<Vec<(Target, u32)>> = free.iter().map(|&a| self.options(a, targets, costs, free_only)).collect();
        // What the creatures from each point on could obey, each its best attack.
        let mut ceiling_after = vec![0u32; free.len() + 1];
        for i in (0..free.len()).rev() {
            ceiling_after[i] = ceiling_after[i + 1] + options[i].first().map_or(0, |o| o.1);
        }
        let used: Vec<u32> = self.limits.iter().map(|&(t, _)| count_u32(fixed.iter().filter(|x| x.1 == t).count())).collect();
        if used.iter().zip(&self.limits).any(|(&n, &(_, most))| n > most) {
            return None;
        }
        struct Search<'a> {
            rules: &'a AttackRules,
            options: &'a [Vec<(Target, u32)>],
            ceiling_after: &'a [u32],
            /// Attacks placed at each of `rules.limits`, by the same index.
            used: Vec<u32>,
            /// What the search stops at, from the free creatures alone: `enough` less what `fixed` obeys.
            enough: Option<u32>,
            chosen: Vec<Option<Target>>,
            best: Option<(u32, Vec<Option<Target>>)>,
        }
        impl Search<'_> {
            fn go(&mut self, i: usize, obeyed: u32) {
                if self.best.as_ref().is_some_and(|(n, _)| self.enough.is_some_and(|e| *n >= e)) {
                    return;
                }
                if i == self.options.len() {
                    if self.best.as_ref().is_none_or(|(n, _)| obeyed > *n) {
                        self.best = Some((obeyed, self.chosen.clone()));
                    }
                    return;
                }
                let reach = obeyed + self.ceiling_after[i];
                if self.enough.is_some_and(|e| reach < e) || self.best.as_ref().is_some_and(|(n, _)| reach <= *n) {
                    return;
                }
                for &(t, g) in &self.options[i] {
                    let capped = self.rules.limits.iter().position(|&(x, _)| x == t);
                    if let Some(k) = capped {
                        if self.used[k] >= self.rules.limits[k].1 {
                            continue;
                        }
                        self.used[k] += 1;
                    }
                    self.chosen[i] = Some(t);
                    self.go(i + 1, obeyed + g);
                    if let Some(k) = capped {
                        self.used[k] -= 1;
                    }
                    if self.best.as_ref().is_some_and(|(n, _)| self.enough.is_some_and(|e| *n >= e)) {
                        return;
                    }
                }
                // This creature attacks nothing.
                self.chosen[i] = None;
                self.go(i + 1, obeyed);
            }
        }
        let fixed_gain = self.obeyed(fixed);
        let mut search = Search {
            rules: self,
            options: &options,
            ceiling_after: &ceiling_after,
            used,
            enough: enough.map(|n| n.saturating_sub(fixed_gain)),
            chosen: vec![None; free.len()],
            best: None,
        };
        search.go(0, 0);
        let (obeyed, chosen) = search.best?;
        let mut declaration = fixed.to_vec();
        declaration.extend(free.iter().zip(&chosen).filter_map(|(&a, t)| t.map(|t| (a, t))));
        Some((fixed_gain + obeyed, declaration))
    }

    /// The most requirements any declaration of `eligible` attacking among `targets` obeys without paying a cost (CR
    /// 508.1d), with one that does (`search`).
    pub fn solve(&self, eligible: &[ObjectId], targets: &[Target], costs: &[AttackCost]) -> (u32, Vec<(ObjectId, Target)>) {
        self.search(eligible, targets, costs, true, &[], None).unwrap_or_default()
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
        let addable = if addable && mendable {
            // A cap is a budget several creatures share (CR 508.1c), so an attack may be added where that target has room
            // and some declaration extending this one with it still obeys the most (CR 508.1d), paying if need be.
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
                    let reaches = self.search(&rest, targets, costs, false, &fixed, Some(self.most)).is_some_and(|(n, _)| n >= self.most);
                    if reaches {
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
        AttackVerdict { complete, addable, issue: found, cost: AttackCost::total(costs, attacks), free: self.most == 0 && self.limits.is_empty() }
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
        assert_eq!(r.suggested.iter().filter(|x| x.1 == b).count(), 1);
        assert!(r.judge(&[U, V], &targets, &[], &r.suggested, false).complete);
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
}
