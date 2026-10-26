//! Paying for a spell or ability without a policy step (docs/ai-player-plan.md, "Answers"): mana payment is not a
//! decision the policy learns. `auto_pay` is the scenario runner's payment (`cantrip_engine::pay`), told what a line says
//! (`PayPlan`); `pay` is what the AI player pays with, `auto_pay` first and then the mana abilities a payment needs.

use std::collections::VecDeque;

use cantrip_engine::*;

pub use cantrip_engine::pay::{PayPlan, Spend, auto_pay, pays};

/// The mana types an object's mana ability adds of fixed kinds (`ActivatedDef::adds`): what an option the engine offers
/// with no `mana` of its own (a basic land's, each of Plateau's two) yields. Empty where the ability is not found.
fn fixed_yield(game: &Game, option: &ActionOption) -> Vec<Mana> {
    let Some(source) = option.source else {
        return Vec::new();
    };
    if option.index >= PLAYER_MANA_ABILITY {
        return Vec::new();
    }
    let Some(ability) = game.chars(source).abilities().nth(option.index as usize) else {
        return Vec::new();
    };
    let Some(def) = game.activated_ability(ability) else {
        return Vec::new();
    };
    Mana::ALL
        .iter()
        .copied()
        .filter(|&m| def.adds.0[m as usize] > 0)
        .collect()
}

/// The mana a mana ability option adds that pays something of `owed`, as the action that adds it; `None` if it adds
/// nothing owed. A choice of color takes the first color owed; an option that names no mana is judged by what its
/// ability adds (`fixed_yield`).
fn mana_action(game: &Game, option: &ActionOption, owed: &ManaCost) -> Option<Action> {
    if option.kind != ActionKind::ActivateAbility
        || option.targets_required > 0 && !option.targets_optional
    {
        return None;
    }
    let mut action = Action::of(option);
    if !option.color_choices.is_empty() {
        let colors = [
            Color::White,
            Color::Blue,
            Color::Black,
            Color::Red,
            Color::Green,
        ];
        let owed_color = colors
            .iter()
            .copied()
            .filter(|&c| option.color_choices.has(c))
            .find(|&c| pays(owed, Mana::ALL[c as usize]));
        let color = owed_color.or_else(|| {
            colors
                .iter()
                .copied()
                .find(|&c| option.color_choices.has(c))
        })?;
        action.color = Some(color);
        if option.mana == Some(Mana::C) {
            action.mana = None;
        }
        return Some(action);
    }
    match option.mana {
        Some(m) if pays(owed, m) => Some(action),
        Some(_) => None,
        None => fixed_yield(game, option)
            .into_iter()
            .any(|m| pays(owed, m))
            .then_some(action),
    }
}

/// The kinds of mana a mana ability option could add: its colors to choose, the mana it names, what its ability adds.
fn kinds(game: &Game, option: &ActionOption) -> Vec<Mana> {
    let colors = [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ];
    let mut kinds: Vec<Mana> = colors
        .iter()
        .filter(|&&c| option.color_choices.has(c))
        .map(|&c| Mana::ALL[c as usize])
        .collect();
    kinds.extend(option.mana);
    kinds.extend(fixed_yield(game, option));
    kinds.sort_by_key(|&m| m as usize);
    kinds.dedup();
    kinds
}

/// Each source in `sources` (the kinds of mana it could add) given to one of `symbols` (the kinds each could be paid
/// with), no source twice: the sources, in the order of the symbols, with the mana each pays. `None` if there is none.
fn cover(
    symbols: &[Vec<Mana>],
    sources: &[Vec<Mana>],
    used: &mut Vec<bool>,
) -> Option<Vec<(usize, Mana)>> {
    let Some((first, rest)) = symbols.split_first() else {
        return Some(Vec::new());
    };
    for (i, kinds) in sources.iter().enumerate() {
        if used[i] {
            continue;
        }
        let Some(&m) = first.iter().find(|m| kinds.contains(m)) else {
            continue;
        };
        used[i] = true;
        if let Some(mut more) = cover(rest, sources, used) {
            more.insert(0, (i, m));
            return Some(more);
        }
        used[i] = false;
    }
    None
}

/// The mana ability to activate next towards `owed`. The colored and hybrid symbols owed are matched to sources first,
/// each its own source (`cover`), and the first of them is tapped for its symbol: so the generic part is never paid with
/// the only source of a color still owed (an Island tapped for {1} of {1}{U}) and a dual land is not spent on a color a
/// basic land adds. With only generic mana owed, or no matching, the first ability that adds anything owed.
fn tap(game: &Game, options: &[ActionOption], owed: &ManaCost) -> Option<Action> {
    let candidates: Vec<(&ActionOption, Vec<Mana>)> = options
        .iter()
        .filter(|o| mana_action(game, o, owed).is_some())
        .map(|o| (o, kinds(game, o)))
        .collect();
    let mut symbols: Vec<Vec<Mana>> = Vec::new();
    for m in Mana::ALL {
        for _ in 0..owed.symbols[m as usize] {
            symbols.push(vec![m]);
        }
    }
    symbols.extend(owed.hybrid.iter().flatten().map(|&(a, b)| vec![a, b]));
    let sources: Vec<Vec<Mana>> = candidates.iter().map(|(_, k)| k.clone()).collect();
    if !symbols.is_empty()
        && let Some(plan) = cover(&symbols, &sources, &mut vec![false; sources.len()])
        && let Some(&(i, m)) = plan.first()
    {
        let option = candidates[i].0;
        let mut action = Action::of(option);
        if !option.color_choices.is_empty() {
            if m == Mana::C {
                action.color = None;
            } else {
                action.color = Some(
                    [
                        Color::White,
                        Color::Blue,
                        Color::Black,
                        Color::Red,
                        Color::Green,
                    ][m as usize],
                );
                if option.mana == Some(Mana::C) {
                    action.mana = None;
                }
            }
        }
        return Some(action);
    }
    options.iter().find_map(|o| mana_action(game, o, owed))
}

/// What the AI player pays a payment with: for `PayCosts` (announcing `x` where it is being announced), `auto_pay`,
/// then a mana ability that adds mana still owed; for `UnlessPays`, X raised or lowered to `x`, then paying once the
/// pool covers it, a mana ability while it does not. `None` where it cannot pay: the caller cancels or declines.
pub fn pay(game: &Game, request: &Request, x: Option<u16>) -> Option<Answer> {
    match &request.kind {
        RequestKind::PayCosts {
            remaining, options, ..
        } => {
            let mut plan = PayPlan::new(x, VecDeque::new());
            if let Ok(Some(answer)) = auto_pay(game, request, &mut plan) {
                return Some(answer);
            }
            tap(game, options, remaining).map(Answer::Act)
        }
        RequestKind::UnlessPays {
            remaining,
            options,
            x: now,
            ..
        } => {
            let offers = |kind: ActionKind| {
                options
                    .iter()
                    .find(|o| o.kind == kind)
                    .map(|o| Answer::Act(Action::of(o)))
            };
            if let (Some(now), Some(want)) = (*now, x) {
                if now < want
                    && let Some(a) = offers(ActionKind::IncreaseX)
                {
                    return Some(a);
                }
                if now > want
                    && let Some(a) = offers(ActionKind::DecreaseX)
                {
                    return Some(a);
                }
            }
            if remaining.is_zero() {
                if let Some(a) = offers(ActionKind::PayCost) {
                    return Some(a);
                }
            }
            tap(game, options, remaining)
                .map(Answer::Act)
                .or_else(|| offers(ActionKind::PayCost))
        }
        _ => None,
    }
}
