//! Judging a declaration of blockers (CR 509.1a-c): one validator for every restriction and requirement a declaration
//! answers to, from the rules the question holds (`BlockRequirements`). `BlockRequirements::judge` says whether a declaration
//! may be made as it is (`BlockVerdict::complete`), why not (`BlockVerdict::issue`), and which blocks may be added to it on
//! the way to one that may (`BlockVerdict::addable`). The core checks every answer with it, the scenario harness reads its
//! issue, and the host gives its verdict to a client as its player declares (`Check`), so no client works the rules out.
//!
//! **"Addable" means that some legal declaration extends the declaration with the block** - not that the declaration with
//! it is legal now: one blocker of a menace creature is no declaration, two are, and a declaration short of a requirement
//! (CR 509.1c) is completed only by adding the block it requires. A block whose blocker already blocks, or that no legal
//! declaration has alongside the blocks declared, is not addable; taking a block back out is always allowed.
//!
//! `solve` is the one search: the most requirements a declaration can obey (`most_requirements` in the core, asked once
//! as the question is made), and whether a declaration that obeys that many extends given blocks (each addable block).

use crate::{BlockRequirements, ObjectId, count_u32};

/// What the rules make of a declaration of blockers.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BlockVerdict {
    /// It may be declared as it is: every block allowed, each blocker blocking one creature, each attacker blocked by as
    /// many creatures as it may be, and as many requirements obeyed as any declaration can (CR 509.1b-c).
    pub complete: bool,
    /// The blocks that some legal declaration extending it has, among the blocks not declared (empty unless asked for).
    pub addable: Vec<(ObjectId, ObjectId)>,
    /// Why it may not be declared as it is; none when it may.
    pub issue: Option<BlockIssue>,
}

/// Why a declaration of blockers may not be made as it is, as a caption reads it: `creature`, then `words`, then `other`
/// where there is one ("Grizzly Bears" "must block" "Ox"; "Ox" "can't be blocked except by two or more creatures"), with
/// the rule it breaks.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockIssue {
    pub creature: ObjectId,
    pub words: String,
    pub other: Option<ObjectId>,
    /// The rule, as the Comprehensive Rules number it ("509.1c").
    pub rule: &'static str,
}

fn creatures(n: u32) -> String {
    const WORDS: [&str; 10] = ["no", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];
    let number = WORDS.get(n as usize).map_or_else(|| n.to_string(), |w| (*w).to_string());
    if n == 1 { format!("{number} creature") } else { format!("{number} creatures") }
}

impl BlockRequirements {
    /// What the rules make of `blocks`, a declaration among the `allowed` blocks (CR 509.1b-c); with `addable`, which blocks
    /// may be added on the way to a legal one too. Without it the verdict is a walk over the blocks, cheap enough to check
    /// every answer with.
    pub fn judge(&self, allowed: &[(ObjectId, ObjectId)], blocks: &[(ObjectId, ObjectId)], addable: bool) -> BlockVerdict {
        let (issue, extendable) = match self.issue(allowed, blocks) {
            Some((issue, extendable)) => (Some(issue), extendable),
            None => (None, true),
        };
        let complete = issue.is_none();
        let addable = if addable && extendable {
            allowed
                .iter()
                .copied()
                .filter(|&(b, a)| {
                    !blocks.iter().any(|x| x.0 == b)
                        && count_u32(blocks.iter().filter(|x| x.1 == a).count()) < self.limits_of(a).1
                        && {
                            let mut more = blocks.to_vec();
                            more.push((b, a));
                            self.issue(allowed, &more).is_none() || self.solve(allowed, &more, Some(self.most)).is_some()
                        }
                })
                .collect()
        } else {
            Vec::new()
        };
        BlockVerdict { complete, addable, issue }
    }

    /// The first thing that keeps `blocks` from being declared as it is, if anything does, and whether blocks added to it could
    /// mend it: not past a block that is not allowed, a creature blocking twice or an attacker over its most.
    fn issue(&self, allowed: &[(ObjectId, ObjectId)], blocks: &[(ObjectId, ObjectId)]) -> Option<(BlockIssue, bool)> {
        let issue = |creature, words: String, other, rule, mendable| Some((BlockIssue { creature, words, other, rule }, mendable));
        for (i, &(b, a)) in blocks.iter().enumerate() {
            if blocks[..i].iter().any(|x| x.0 == b) {
                return issue(b, "can block only one creature".into(), None, "509.1a", false);
            }
            if !allowed.contains(&(b, a)) {
                return issue(b, "can't block".into(), Some(a), "509.1b", false);
            }
        }
        for (i, &(_, a)) in blocks.iter().enumerate() {
            if blocks[..i].iter().any(|x| x.1 == a) {
                continue;
            }
            let n = count_u32(blocks.iter().filter(|x| x.1 == a).count());
            let (least, most) = self.limits_of(a);
            if n > most {
                return issue(a, format!("can't be blocked by more than {}", creatures(most)), None, "509.1b", false);
            }
            if n < least {
                return issue(a, format!("can't be blocked except by {} or more", creatures(least)), None, "509.1b", true);
            }
        }
        if self.obeyed(blocks) < self.most {
            // A requirement left unobeyed by a creature that obeys none, so the caption names a creature that can still obey
            // it; any unobeyed one where every such creature obeys another.
            let unobeyed = |r: &&crate::RequiredBlock| !blocks.iter().any(|&(b, x)| self.obeys(r, b, x));
            let obeys_any = |c: ObjectId| blocks.iter().any(|&(b, x)| b == c && self.required.iter().any(|r| self.obeys(r, b, x)));
            let r = self.required.iter().filter(unobeyed).find(|r| !obeys_any(r.blocker)).or_else(|| self.required.iter().find(unobeyed))?;
            return issue(r.blocker, "must block".into(), r.attacker, "509.1c", true);
        }
        None
    }

    /// The declaration extending `fixed` with blocks among `allowed`, by creatures `fixed` does not have block, that keeps
    /// every attacker it blocks within its limits and obeys the most requirements (CR 509.1c), with how many; with `enough`,
    /// the first found that obeys at least that many. `None` when there is none.
    ///
    /// Only blocks of an attacker a requirement names, one a required creature may block, or one `fixed` blocks can obey a
    /// requirement or make up an attacker's fewest blockers, so only those are searched: the blockers with a requirement
    /// first, their required blocks tried first, and a branch cut when it cannot beat the best found (or reach `enough`) or
    /// leaves an attacker short of blockers no remaining creature can make up.
    pub fn solve(&self, allowed: &[(ObjectId, ObjectId)], fixed: &[(ObjectId, ObjectId)], enough: Option<u32>) -> Option<(u32, Vec<(ObjectId, ObjectId)>)> {
        let mut targets: Vec<ObjectId> = Vec::new();
        let add = |x: ObjectId, targets: &mut Vec<ObjectId>| {
            if !targets.contains(&x) {
                targets.push(x);
            }
        };
        for &(_, a) in fixed {
            add(a, &mut targets);
        }
        for r in &self.required {
            for &(b, a) in allowed {
                if self.obeys(r, b, a) && !fixed.iter().any(|f| f.0 == b) {
                    add(a, &mut targets);
                }
            }
        }
        let limits: Vec<(u32, u32)> = targets.iter().map(|&a| self.limits_of(a)).collect();
        let gain = |b: ObjectId, a: ObjectId| count_u32(self.required.iter().filter(|r| self.obeys(r, b, a)).count());
        let mut counts = vec![0u32; targets.len()];
        let mut fixed_gain = 0;
        for &(b, a) in fixed {
            if let Some(t) = targets.iter().position(|&x| x == a) {
                counts[t] += 1;
            }
            fixed_gain += gain(b, a);
        }
        // Each other creature that could block one of `targets`, with the attackers it may block and how many requirements
        // each block obeys, the most first.
        let mut blockers: Vec<(ObjectId, Vec<(usize, u32)>)> = Vec::new();
        for &(b, a) in allowed {
            if fixed.iter().any(|f| f.0 == b) {
                continue;
            }
            let Some(t) = targets.iter().position(|&x| x == a) else { continue };
            let g = gain(b, a);
            match blockers.iter_mut().find(|e| e.0 == b) {
                Some(e) => e.1.push((t, g)),
                None => blockers.push((b, vec![(t, g)])),
            }
        }
        for (_, options) in &mut blockers {
            options.sort_by_key(|&(_, g)| std::cmp::Reverse(g));
        }
        blockers.sort_by_key(|(_, options)| std::cmp::Reverse(options.first().map_or(0, |o| o.1)));
        // The most the blockers from each point on could still obey (each its best block), and how many of them could block
        // each target.
        let mut required_after = vec![0u32; blockers.len() + 1];
        let mut able_after = vec![vec![0u32; targets.len()]; blockers.len() + 1];
        for i in (0..blockers.len()).rev() {
            required_after[i] = required_after[i + 1] + blockers[i].1.first().map_or(0, |o| o.1);
            able_after[i] = able_after[i + 1].clone();
            for &(t, _) in &blockers[i].1 {
                able_after[i][t] += 1;
            }
        }

        struct Search<'a> {
            blockers: &'a [(ObjectId, Vec<(usize, u32)>)],
            limits: &'a [(u32, u32)],
            required_after: &'a [u32],
            able_after: &'a [Vec<u32>],
            enough: Option<u32>,
            counts: Vec<u32>,
            chosen: Vec<Option<usize>>,
            best: Option<(u32, Vec<Option<usize>>)>,
        }
        impl Search<'_> {
            fn done(&self) -> bool {
                self.enough.is_some() && self.best.is_some()
            }

            fn go(&mut self, i: usize, obeyed: u32) {
                let reach = obeyed + self.required_after[i];
                match self.enough {
                    Some(n) if reach < n => return,
                    None if self.best.as_ref().is_some_and(|(n, _)| reach <= *n) => return,
                    _ => {}
                }
                // An attacker blocked by fewer than it may be needs that many more of the creatures still to come.
                if self.counts.iter().zip(self.limits).enumerate().any(|(t, (&n, &(least, _)))| n > 0 && n < least && self.able_after[i][t] < least - n) {
                    return;
                }
                if i == self.blockers.len() {
                    self.best = Some((obeyed, self.chosen.clone()));
                    return;
                }
                for k in 0..self.blockers[i].1.len() {
                    let (t, gain) = self.blockers[i].1[k];
                    if self.counts[t] >= self.limits[t].1 {
                        continue;
                    }
                    self.counts[t] += 1;
                    self.chosen[i] = Some(t);
                    self.go(i + 1, obeyed + gain);
                    self.counts[t] -= 1;
                    if self.done() {
                        return;
                    }
                }
                self.chosen[i] = None;
                self.go(i + 1, obeyed);
            }
        }
        if counts.iter().zip(&limits).any(|(&n, &(_, most))| n > most) {
            return None;
        }
        let mut search = Search {
            blockers: &blockers,
            limits: &limits,
            required_after: &required_after,
            able_after: &able_after,
            enough: enough.map(|n| n.saturating_sub(fixed_gain)),
            counts,
            chosen: vec![None; blockers.len()],
            best: None,
        };
        search.go(0, 0);
        let (obeyed, chosen) = search.best?;
        let mut declaration = fixed.to_vec();
        declaration.extend(blockers.iter().zip(chosen).filter_map(|((b, _), t)| t.map(|t| (*b, targets[t]))));
        Some((obeyed + fixed_gain, declaration))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequiredBlock;

    const A: ObjectId = ObjectId(1);
    const B: ObjectId = ObjectId(2);
    const X: ObjectId = ObjectId(10);
    const Y: ObjectId = ObjectId(11);

    fn rules(required: Vec<RequiredBlock>, limits: Vec<(ObjectId, u32, u32)>, allowed: &[(ObjectId, ObjectId)]) -> BlockRequirements {
        let mut r = BlockRequirements { required, limits, ..Default::default() };
        if let Some((most, suggested)) = r.solve(allowed, &[], None) {
            r.most = most;
            r.suggested = if most > 0 { suggested } else { Vec::new() };
        }
        r
    }

    /// A requirement to block is one requirement, obeyed by blocking any attacker the creature may block.
    #[test]
    fn a_creature_that_must_block_obeys_by_blocking_any_attacker() {
        let allowed = [(X, A), (X, B)];
        let r = rules(vec![RequiredBlock { blocker: X, attacker: None }], vec![(A, 1, u32::MAX), (B, 1, u32::MAX)], &allowed);
        assert_eq!(r.most, 1);
        assert!(r.judge(&allowed, &[(X, A)], false).complete && r.judge(&allowed, &[(X, B)], false).complete);
        let none = r.judge(&allowed, &[], true);
        assert!(!none.complete);
        assert_eq!(none.issue.as_ref().map(|i| (i.creature, i.rule)), Some((X, "509.1c")));
        assert_eq!(none.addable, vec![(X, A), (X, B)]);
    }

    /// A required blocker of a menace creature alone obeys nothing: with a second creature able to join it, the requirement
    /// is obeyed by both blocking; adding the first block is addable though the declaration with it alone is not legal.
    #[test]
    fn menace_needs_a_second_blocker_for_the_requirement() {
        let allowed = [(X, A), (Y, A)];
        let r = rules(vec![RequiredBlock { blocker: X, attacker: None }], vec![(A, 2, u32::MAX)], &allowed);
        assert_eq!(r.most, 1);
        let one = r.judge(&allowed, &[(X, A)], true);
        assert!(!one.complete && one.addable == vec![(Y, A)]);
        assert!(r.judge(&allowed, &[(X, A), (Y, A)], false).complete);
        // With no second creature, nothing is required.
        let alone = rules(vec![RequiredBlock { blocker: X, attacker: None }], vec![(A, 2, u32::MAX)], &[(X, A)]);
        assert_eq!(alone.most, 0);
        assert!(alone.judge(&[(X, A)], &[], false).complete);
        assert!(alone.judge(&[(X, A)], &[], true).addable.is_empty(), "no legal declaration has X blocking alone");
    }

    /// A requirement to block any attacker and a Lure-style one to block a given attacker are counted together: where only
    /// one creature can block the lured attacker, the declaration that obeys both has the other block elsewhere.
    #[test]
    fn requirements_to_block_and_to_block_an_attacker_add_up() {
        let allowed = [(X, A), (X, B), (Y, A)];
        let required = vec![RequiredBlock { blocker: X, attacker: None }, RequiredBlock { blocker: Y, attacker: Some(A) }, RequiredBlock { blocker: X, attacker: Some(A) }];
        let r = rules(required, vec![(A, 1, 1), (B, 1, u32::MAX)], &allowed);
        // A can be blocked by one creature: Y on A and X on B obeys two, as does X on A alone (two of X's) - the most is 2.
        assert_eq!(r.most, 2);
        assert!(r.judge(&allowed, &[(Y, A), (X, B)], false).complete);
        assert!(r.judge(&allowed, &[(X, A)], false).complete);
        assert!(!r.judge(&allowed, &[(Y, A)], false).complete);
        assert!(!r.judge(&allowed, &[(X, B)], false).complete);
        let over = r.judge(&allowed, &[(X, A), (Y, A)], true);
        assert!(!over.complete && over.addable.is_empty() && over.issue.is_some_and(|i| i.creature == A));
    }
}
