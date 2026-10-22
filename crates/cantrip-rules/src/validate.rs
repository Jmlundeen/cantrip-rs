//! Whether an answer fits its request: the right kind, and only what was offered. The engine refuses anything else
//! before it touches the game, so a refused answer leaves the request outstanding.

use cantrip_api::select;
use cantrip_api::*;

use crate::{Game, Rejected};

fn no(why: impl Into<String>) -> Result<(), Rejected> {
    Err(Rejected(why.into()))
}

fn distinct<T: PartialEq>(items: &[T]) -> bool {
    items.iter().enumerate().all(|(i, a)| items[..i].iter().all(|b| b != a))
}

fn fits(option: &ActionOption, action: &Action) -> bool {
    if option.kind != action.kind || option.source != action.source || option.index != action.index {
        return false;
    }
    // The permission and the alternative cost are the option's own (CR 601.2b): an action may not name others.
    if option.granted_by != action.granted_by || option.paid_by != action.paid_by {
        return false;
    }
    // Restricted mana is a kind of its own (CR 106.6): spending it is answered with its restriction.
    if option.kind == ActionKind::AssignMana && (option.mana != action.mana || option.restriction != action.restriction) {
        return false;
    }
    match action.color {
        Some(c) => {
            if !option.color_choices.has(c) {
                return false;
            }
        }
        None => {
            // Colorless named for an ability whose sacrificed permanent could add it (CR 106.7): the option says so.
            if option.kind == ActionKind::ActivateAbility && action.mana == Some(Mana::C) {
                if option.mana != Some(Mana::C) {
                    return false;
                }
            } else if !option.color_choices.is_empty() && !(option.names_nothing && action.mana.is_none()) {
                return false;
            }
        }
    }
    let (n, required) = (action.targets.len(), option.targets_required as usize);
    if n > required || (!option.targets_optional && n < required) {
        return false;
    }
    distinct(&action.targets) && action.targets.iter().all(|t| option.target_candidates.contains(t))
}

/// What is wrong with a distribution of looked-at cards, if anything (the C# `CardDistribution.Problem`).
pub fn distribution_problem(cards: &[ObjectId], destinations: &[Destination], placements: &[Vec<ObjectId>]) -> Option<String> {
    if placements.len() != destinations.len() {
        return Some(format!("the answer places cards into {} destinations, but there are {}", placements.len(), destinations.len()));
    }
    let mut placed: Vec<ObjectId> = Vec::new();
    for (i, (d, p)) in destinations.iter().zip(placements).enumerate() {
        let n = p.len();
        if n < d.min as usize || (d.max != u8::MAX && n > d.max as usize) {
            return Some(format!("destination {i} takes from {} to {}, not {n}", d.min, d.max));
        }
        for card in p {
            if placed.contains(card) {
                return Some("a card was placed in more than one destination".into());
            }
            if !d.eligible.contains(card) {
                return Some(format!("a card was placed in destination {i}, which does not accept it"));
            }
            placed.push(*card);
        }
    }
    if placed.len() != cards.len() {
        return Some(format!("every one of the {} cards needs a destination; {} were placed", cards.len(), placed.len()));
    }
    None
}

/// A goaded creature attacks a player other than its goaders if it can (CR 701.15b): whether `attacker` attacking
/// `at` breaks that, some other player being there to attack.
fn goaded_into_its_goader(game: &Game, attacker: ObjectId, at: PlayerId, players: &[PlayerId], costs: &[AttackCost]) -> bool {
    let goaders = |p: PlayerId| game.state.effects.iter().any(|e| e.affected == attacker && e.kind == EffectKind::Goaded && e.controller == p);
    // "If able" never means paying a cost to attack (CR 508.1d).
    goaders(at) && players.iter().any(|&p| !goaders(p) && AttackCost::free(costs, attacker, Target::Player(p)))
}

/// Why the name of `card` may not be chosen for a question asking for a `kind` of card name, or `None` if it may.
/// A name chosen "with certain characteristics" is that of a card that has them (CR 201.4a); a player naming a split card
/// names one of its two names, not both (CR 201.4b, 709.4a). The name of a token or an emblem is no card name unless a card
/// has it too (CR 201.4).
pub fn name_problem(cards: &CardTable, kind: NameKind, card: CardId) -> Option<&'static str> {
    let def = cards.get(card);
    if !cards.card_named[card.0 as usize] {
        return Some("not the name of a card (a token's or an emblem's only)");
    }
    if !kind.allows(def.is(CardType::Land)) {
        return Some(if kind == NameKind::Land { "not a land card name" } else { "a land is not a nonland card name" });
    }
    if def.split.is_some() {
        return Some("name one of a split card's names, not both");
    }
    None
}

/// Why a selection of `n` cards a rule refuses is refused, in its words: the count it takes, and what its parts ask.
fn refusal(rule: &Selection, n: usize) -> String {
    let wanted = match rule.count {
        Count::Exactly(c) => format!("{c}"),
        Count::UpTo(c) => format!("up to {c}"),
        Count::AnyNumber => "any number".to_string(),
        Count::AsManyAsCan(c) => format!("{c}, or as many as can be"),
    };
    let words = rule.words();
    let asks = if words.is_empty() { String::new() } else { format!(", {}", words.join(", ")) };
    format!("{n} chosen, {wanted} wanted{asks}, as the question's rule allows")
}

/// What the rule of a question that picks a set of cards says of `selected`, given as the cards it picks among: whether
/// they are an answer and which of the cards may be added to them (`Verdict`). The question a client checks a partial
/// selection against as its player picks (the host's `Check`): a choice of cards and a library search judge by their rule
/// (`Selection`, the same check `check` makes of an answer), and a cost's tapping for power, collecting evidence and
/// crafting by theirs. `None` for any other question.
pub fn selection_verdict(game: &Game, request: &Request, selected: &[ObjectId]) -> Option<(Vec<ObjectId>, Verdict)> {
    let unchosen = |cards: &[ObjectId]| Bits::of(cards.len(), |i| !selected.contains(&cards[i]));
    let among = |cards: &[ObjectId]| distinct(selected) && selected.iter().all(|c| cards.contains(c));
    let source = request.kind.source().unwrap_or(ObjectId::NONE);
    Some(match &request.kind {
        RequestKind::ChooseCards { cards, rule, .. } | RequestKind::SearchLibrary { library: cards, rule, .. } => {
            let verdict = select::judge(game, rule, &Choice { chooser: request.player, source, candidates: cards, selected });
            (cards.clone(), verdict)
        }
        // Harmonize's creature is up to one (CR 702.180a).
        RequestKind::TapForPower { purpose: Purpose::TapToReduce, cards, .. } => {
            let addable = if selected.is_empty() { unchosen(cards) } else { Bits::new(cards.len(), false) };
            (cards.clone(), Verdict { complete: among(cards) && selected.len() <= 1, addable })
        }
        // Crew and saddle (CR 702.122a, 702.171a): any of them, with total power `minimum` or more; signed, a negative power
        // taking from it (CR 107.1b).
        RequestKind::TapForPower { cards, powers, minimum, .. } => {
            let total: i32 = selected.iter().filter_map(|c| cards.iter().position(|x| x == c)).map(|i| i32::from(powers[i])).sum();
            (cards.clone(), Verdict { complete: among(cards) && total >= i32::from(*minimum), addable: unchosen(cards) })
        }
        // Collect evidence (CR 701.59a): total mana value `minimum` or more.
        RequestKind::CollectEvidence { cards, values, minimum, .. } => {
            let total: u32 = selected.iter().filter_map(|c| cards.iter().position(|x| x == c)).map(|i| u32::from(values[i])).sum();
            (cards.clone(), Verdict { complete: among(cards) && total >= u32::from(*minimum), addable: unchosen(cards) })
        }
        // Craft's materials (CR 702.167a): `minimum` to `maximum` of them.
        RequestKind::Craft { cards, minimum, maximum, .. } => {
            let n = selected.len();
            let addable = if n < *maximum as usize { unchosen(cards) } else { Bits::new(cards.len(), false) };
            (cards.clone(), Verdict { complete: among(cards) && n >= *minimum as usize && n <= *maximum as usize, addable })
        }
        _ => return None,
    })
}

/// Whether `answer` fits `request` (exported as `check_answer`, for a scenario that shows an answer is refused).
pub fn check(game: &Game, request: &Request, answer: &Answer) -> Result<(), Rejected> {
    if *answer == Answer::Cancel {
        return if request.may_cancel { Ok(()) } else { no("this cannot be cancelled") };
    }
    match (&request.kind, answer) {
        (RequestKind::Priority { .. }, Answer::Pass) => Ok(()),
        (RequestKind::Priority { options }, Answer::Act(action))
        | (RequestKind::PayCosts { options, .. }, Answer::Act(action))
        | (RequestKind::UnlessPays { options, .. }, Answer::Act(action)) => {
            if options.iter().any(|o| fits(o, action)) { Ok(()) } else { no("that action is not one of the options") }
        }
        (RequestKind::PayCosts { announcing: true, most, .. }, Answer::Announce(value)) => {
            if value <= most { Ok(()) } else { no(format!("{value} announced, {most} at most (CR 601.2b)")) }
        }
        (RequestKind::ChooseCards { cards: candidates, rule, .. }, Answer::Cards(cards)) | (RequestKind::SearchLibrary { library: candidates, rule, .. }, Answer::Cards(cards)) => {
            // The question's own rule judges the answer, as it judged each pick (`selection_verdict`).
            let source = request.kind.source().unwrap_or(ObjectId::NONE);
            if !distinct(cards) || !cards.iter().all(|c| candidates.contains(c) && rule.listed_card(*c)) {
                no("not among the cards that may be chosen")
            } else if !select::answers(game, rule, request.player, source, candidates, cards) {
                no(refusal(rule, cards.len()))
            } else {
                Ok(())
            }
        }
        (RequestKind::ChooseTarget { candidates, least, count, total, .. }, Answer::Targets(targets)) => {
            let n = targets.len();
            if !distinct(targets) || !targets.iter().all(|t| candidates.contains(t)) {
                no("not among the candidates")
            } else if n > *count as usize || n < *least as usize {
                no(format!("{n} chosen, {least} to {count} wanted (CR 601.2c)"))
            } else if let Some(t) = total.as_ref().filter(|t| !t.allows(candidates, targets)) {
                no(format!("total mana value {}, {} or less wanted (CR 601.2c, 202.3)", t.of(candidates, targets), t.most))
            } else {
                Ok(())
            }
        }
        (RequestKind::ChooseModes { eligible, count, at_most, repeat, .. }, Answer::Modes(modes)) => {
            let most = (*at_most).max(*count) as usize;
            if modes.len() >= *count as usize
                && modes.len() <= most
                && (*repeat || distinct(modes))
                && modes.iter().all(|m| eligible.contains(m))
            {
                Ok(())
            } else {
                no("not a choice of the modes offered")
            }
        }
        (RequestKind::ChooseColors { choices, count, .. }, Answer::Colors(colors)) => {
            if colors.len() == usize::from(*count) && colors.iter().all(|c| choices.has(*c)) {
                Ok(())
            } else {
                no("not a choice of the colors offered")
            }
        }
        (RequestKind::May { .. }, Answer::Yes | Answer::No) => Ok(()),
        (RequestKind::ChoosePlayer { candidates, .. }, Answer::Player(p)) => {
            if candidates.contains(p) { Ok(()) } else { no("not a player who may be chosen") }
        }
        (RequestKind::ChooseDividedTargets { candidates, total, max_targets, optional, .. }, Answer::Divided(shares)) => {
            // Added up in a u64, so no number of u32 shares overflows it.
            let sum: u64 = shares.iter().map(|s| u64::from(s.1)).sum();
            let targets: Vec<Target> = shares.iter().map(|s| s.0).collect();
            if !distinct(&targets) || !targets.iter().all(|t| candidates.contains(t)) {
                no("not among the candidates")
            } else if *total == 0 {
                if shares.is_empty() { Ok(()) } else { no("there is nothing to divide") }
            } else if shares.is_empty() {
                if *optional { Ok(()) } else { no("the total must be divided among at least one target (CR 601.2d)") }
            } else if (*max_targets > 0 && shares.len() > *max_targets as usize) || shares.iter().any(|s| s.1 < 1) || sum != u64::from(*total) {
                no(format!("the shares must each be at least 1 and add up to {total}"))
            } else {
                Ok(())
            }
        }
        (RequestKind::TapForPower { purpose: Purpose::TapToReduce, .. }, Answer::Cards(chosen)) if chosen.len() > 1 => {
            no("up to one creature may be tapped (CR 702.180a)")
        }
        (RequestKind::TapForPower { cards, powers, minimum, .. }, Answer::Cards(chosen)) => {
            // Signed: a creature with negative power takes from the total (CR 107.1b).
            let total: i32 = chosen.iter().filter_map(|c| cards.iter().position(|x| x == c)).map(|i| i32::from(powers[i])).sum();
            if !distinct(chosen) || !chosen.iter().all(|c| cards.contains(c)) {
                no("not among the creatures that may be tapped")
            } else if total < i32::from(*minimum) {
                no(format!("total power {total}, {minimum} needed"))
            } else {
                Ok(())
            }
        }
        (RequestKind::Craft { cards, minimum, maximum, .. }, Answer::Cards(chosen)) => {
            let n = chosen.len();
            if !distinct(chosen) || !chosen.iter().all(|c| cards.contains(c)) {
                no("not among the materials that may be exiled")
            } else if n < *minimum as usize || n > *maximum as usize {
                no(format!("{n} chosen, {minimum} to {maximum} wanted (CR 702.167a)"))
            } else {
                Ok(())
            }
        }
        (RequestKind::RemoveCounters { pools, total, up_to, .. }, Answer::Counters(taken)) => {
            let mut sum = 0u32;
            for (i, &(object, kind, n)) in taken.iter().enumerate() {
                let Some(pool) = pools.iter().find(|p| p.object == object && p.kind == kind) else {
                    return no("not a kind of counter on a permanent the cost may take from");
                };
                if n == 0 || n > pool.count {
                    return no(format!("{n} {} counters taken, {} there", kind.name(), pool.count));
                }
                if taken[..i].iter().any(|&(o, k, _)| o == object && k == kind) {
                    return no("the same counters named twice");
                }
                sum += n as u32;
            }
            if *up_to {
                // "Up to" (CR 608.2d): none at all is a choice, more than the number is not.
                if sum > *total as u32 { no(format!("{sum} counters removed, up to {total} allowed")) } else { Ok(()) }
            } else if sum != *total as u32 {
                no(format!("{sum} counters removed, {total} wanted (CR 118.3)"))
            } else {
                Ok(())
            }
        }
        (RequestKind::CollectEvidence { cards, values, minimum, .. }, Answer::Cards(chosen)) => {
            let total: u32 = chosen.iter().filter_map(|c| cards.iter().position(|x| x == c)).map(|i| values[i] as u32).sum();
            if !distinct(chosen) || !chosen.iter().all(|c| cards.contains(c)) {
                no("not among the cards that may be exiled")
            } else if total < *minimum as u32 {
                no(format!("total mana value {total}, {minimum} needed"))
            } else {
                Ok(())
            }
        }
        (RequestKind::Mulligan { .. }, Answer::Yes | Answer::No) => Ok(()),
        (RequestKind::DeclareAttackers { eligible, players, planeswalkers, battles, requirements, costs, .. }, Answer::Attack(attacks)) => {
            let ids: Vec<ObjectId> = attacks.iter().map(|a| a.0).collect();
            let legal = |t: &Target| match t {
                Target::Player(p) => players.contains(p),
                Target::Object(b) => battles.contains(b) || planeswalkers.contains(b),
                _ => false,
            };
            if !distinct(&ids) || !attacks.iter().all(|(a, t)| eligible.contains(a) && legal(t)) {
                no("not a legal declaration of attackers")
            } else if !players.is_empty() && !requirements.iter().all(|r| ids.contains(r)) {
                no("a creature that attacks each combat if able is not attacking")
            } else if attacks.iter().any(|&(a, t)| {
                // CR 508.1d: a creature required to attack a certain player attacks that player if able - able without
                // paying a cost to attack, which is never required.
                game.state.effects.iter().any(|e| {
                    matches!(e.kind, EffectKind::MustAttackPlayer(p)
                        if e.affected == a && players.contains(&p) && t != Target::Player(p) && AttackCost::free(costs, a, Target::Player(p)))
                })
            }) {
                no("a creature that must attack a certain player is not attacking them")
            } else if attacks.iter().any(|&(a, t)| matches!(t, Target::Player(p) if goaded_into_its_goader(game, a, p, players, costs))) {
                no("a goaded creature attacks a player other than its goader if able (CR 701.15b)")
            } else {
                Ok(())
            }
        }
        (RequestKind::AssignCombatDamage { divisions }, Answer::Shares(shares)) => {
            for d in divisions {
                let mine: Vec<&(ObjectId, Option<ObjectId>, u16)> = shares.iter().filter(|s| s.0 == d.source).collect();
                let total: u16 = mine.iter().map(|s| s.2).sum();
                if total != d.damage {
                    return no("a creature's combat damage must all be assigned, and no more (CR 510.1a)");
                }
                for s in &mine {
                    match s.1 {
                        Some(c) if !d.creatures.contains(&c) => return no("damage assigned to a creature not blocking it"),
                        None if !d.may_assign_to_defender => return no("only trample assigns damage past its blockers"),
                        _ => {}
                    }
                }
                let to_defender: u16 = mine.iter().filter(|s| s.1.is_none()).map(|s| s.2).sum();
                if to_defender > 0 {
                    for (i, &c) in d.creatures.iter().enumerate() {
                        let got: u16 = mine.iter().filter(|s| s.1 == Some(c)).map(|s| s.2).sum();
                        if got < d.lethal[i] {
                            return no("nothing reaches the player until every blocker is assigned lethal damage (CR 702.19b)");
                        }
                    }
                }
            }
            if shares.iter().any(|s| !divisions.iter().any(|d| d.source == s.0)) {
                return no("damage assigned for a creature with nothing to divide");
            }
            Ok(())
        }
        (RequestKind::DeclareBlockers { allowed, requirements, .. }, Answer::Block(blocks)) => {
            let ids: Vec<ObjectId> = blocks.iter().map(|b| b.0).collect();
            if !distinct(&ids) || !blocks.iter().all(|b| allowed.contains(b)) {
                no("not a legal declaration of blockers")
            } else if blocks.iter().any(|&(_, a)| {
                // Menace (CR 702.110b), "except by three or more creatures": blocked by that many or not at all.
                blocks.iter().filter(|b| b.1 == a).count() < game.chars(a).minimum_blockers()
            }) {
                no("a creature can't be blocked by fewer creatures than it requires (CR 509.1b, 702.110b)")
            } else if blocks.iter().any(|&(_, a)| {
                // "Can't be blocked by more than one creature" (CR 509.1b).
                blocks.iter().filter(|b| b.1 == a).count() > game.chars(a).maximum_blockers()
            }) {
                no("a creature can't be blocked by more creatures than it allows (CR 509.1b)")
            } else if requirements.obeyed(blocks) < requirements.most {
                // CR 509.1c: as many requirements obeyed as any declaration that disobeys no restriction obeys.
                no("a creature able to block where an effect requires it to does not (CR 509.1c)")
            } else {
                Ok(())
            }
        }
        (RequestKind::DiscardToHandSize { count, hand }, Answer::Cards(cards)) => {
            if cards.len() == *count as usize && distinct(cards) && cards.iter().all(|c| hand.contains(c)) {
                Ok(())
            } else {
                no("not a discard of the right cards")
            }
        }
        (RequestKind::DistributeCards { cards, destinations, may_shuffle, .. }, Answer::Placements { placements, shuffle }) => {
            if *shuffle {
                return if *may_shuffle { Ok(()) } else { no("shuffling is not allowed here") };
            }
            match distribution_problem(cards, destinations, placements) {
                Some(why) => no(why),
                None => Ok(()),
            }
        }
        (RequestKind::OrderTriggers { sources, .. }, Answer::Order(order)) => {
            let mut seen = order.clone();
            seen.sort();
            if seen == (0..count_u32(sources.len())).collect::<Vec<_>>() { Ok(()) } else { no("not an order of every trigger") }
        }
        (RequestKind::ChooseReplacement { sources, .. }, Answer::Index(i)) => {
            if (*i as usize) < sources.len() { Ok(()) } else { no("not one of the replacement effects offered") }
        }
        (RequestKind::ChooseName { kind, .. }, Answer::Name(name)) => match name.and_then(|card| name_problem(game.cards, *kind, card)) {
            Some(problem) => no(problem),
            None => Ok(()),
        },
        (RequestKind::ChooseSubtype { choices, .. }, Answer::Subtype(s)) => {
            if choices.contains(s) { Ok(()) } else { no("not one of the types offered") }
        }
        (RequestKind::ChooseStartingPlayer { candidates }, Answer::Player(p)) => {
            if candidates.contains(p) { Ok(()) } else { no("not a player who may go first") }
        }
        _ => no("that is not an answer to this question"),
    }
}

/// The cards that may be chosen first for a question that picks a set of cards (`selection_verdict` of none chosen): its
/// eligible cards, for a test or a player reading the question. Empty for any other question.
pub fn first_choices(game: &Game, request: &Request) -> Vec<ObjectId> {
    selection_verdict(game, request, &[]).map_or_else(Vec::new, |(cards, verdict)| verdict.addable.ones().map(|i| cards[i]).collect())
}
