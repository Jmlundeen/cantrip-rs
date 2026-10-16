use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

fn food(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::FOOD, 1)?;
    Ok(())
}

fn artifact_or_creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && {
        let c = q.chars(id);
        c.is(CardType::Artifact) || c.is(CardType::Creature)
    })
}

/// "Target artifact or creature loses all abilities and becomes a green Elk creature with base power and toughness
/// 3/3": for good. The card types are replaced (and the subtypes of the lost ones go with them), Elk is added.
fn elk(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let source = item.object;
    let words = g.words(source);
    g.add_effect(source, target, EffectKind::LoseAllAbilities, Duration::Indefinite);
    g.add_effect(source, target, EffectKind::SetTypes(TypeSet::of(&[CardType::Creature])), Duration::Indefinite);
    g.add_effect(source, target, EffectKind::AddSubtype(words.subtype(Subtype::Elk)), Duration::Indefinite);
    g.add_effect(source, target, EffectKind::SetColors(words.colors(ColorSet::of(Color::Green))), Duration::Indefinite);
    g.add_effect(source, target, EffectKind::SetPt(3, 3), Duration::Indefinite);
    Ok(())
}

fn artifact_or_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    artifact_or_creature(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) == asker.controller)
}

fn small_creature_an_opponent_controls(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE_AN_OPPONENT_CONTROLS.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if q.chars(id).power.unwrap_or(0) <= 3)
}

/// "Exchange control of target artifact or creature you control and target creature an opponent controls with power
/// 3 or less" (CR 701.12): if the whole exchange can't be completed, none of it happens. Both controllers are read
/// before either changes.
fn exchange(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !g.target_legal(item, 0) || !g.target_legal(item, 1) {
        return Ok(());
    }
    let (Target::Object(yours), Target::Object(theirs)) = (item.targets[0], item.targets[1]) else { return Ok(()) };
    g.exchange_control(item.object, yours, theirs);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(2), food, "+2: Create a Food token."),
        ability(loyalty(1), elk, "+1: Target artifact or creature loses all abilities and becomes a green Elk creature with base power and toughness 3/3.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: artifact_or_creature, label: "" }]),
        ability(loyalty(-5), exchange, "\u{2212}5: Exchange control of target artifact or creature you control and target creature an opponent controls with power 3 or less.")
            .targets(&[
                TargetSpec { min: 1, max: 1, filter: artifact_or_creature_you_control, label: "" },
                TargetSpec { min: 1, max: 1, filter: small_creature_an_opponent_controls, label: "" },
            ]),
    ],
    ..oracle::OKO_THIEF_OF_CROWNS
};

pub static EXTRA: &[&CardDef] = &[&tokens::FOOD];
