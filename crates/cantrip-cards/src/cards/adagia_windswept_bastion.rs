use cantrip_api::*;
use cantrip_mechanics::{mana, station, targets};
use cantrip_oracle as oracle;

/// "Target artifact or enchantment you control" (CR 115.1).
fn artifact_or_enchantment_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.controller(id) == asker.controller && {
        let c = q.chars(id);
        c.is(CardType::Artifact) || c.is(CardType::Enchantment)
    })
}

/// "Create a token that's a copy of target artifact or enchantment you control, except it's legendary" (CR 707.2, 707.9b,
/// 111.1): the target's copiable values, its own copy effect's exceptions included, with legendary added. Nothing happens
/// if the target is no longer legal (CR 608.2b).
fn copy_except_legendary(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let values = g.copiable_values(target).legendary();
    g.create_token_copies_of(values, item.controller, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::W), "{T}: Add {W}."),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Planet. Station only as a sorcery.)"),
    ],
    // "This land enters tapped" (CR 614.1c).
    enters: Some(mana::enters_tapped),
    // The 12+ striation (CR 721.2a): Adagia has this ability while it has twelve or more charge counters on it.
    delayed: &[ability(cost("{3}{W}, {T}"), copy_except_legendary, "{3}{W}, {T}: Create a token that's a copy of target artifact or enchantment you control, except it's legendary. Activate only as a sorcery.")
        .targets(&[target(artifact_or_enchantment_you_control)])
        .sorcery_speed()],
    statics: &[static_ability(Layer::Ability, station::charged::<12>, station::has::<0>, "12+ | {3}{W}, {T}: Create a token that's a copy of target artifact or enchantment you control, except it's legendary. Activate only as a sorcery.")],
    ..oracle::ADAGIA_WINDSWEPT_BASTION
};
