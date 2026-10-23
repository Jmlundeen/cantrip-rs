use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Each stage replaces the creature's creature types and base power and toughness (CR 205.1a, 613.1d), where a later effect
/// wins by its timestamp: it loses the creature types it had, an earlier stage's among them, and the new ones follow. Its
/// card types do not change, so an artifact type Mycosynth Lattice gives it stays.
fn become_kithkin(g: &mut dyn Game, source: ObjectId, kind: Subtype, power: i16, toughness: i16) {
    let words = g.words(source);
    // "It becomes a Kithkin [...] with base power and toughness [P]/[T]": one instruction (CR 608.2c).
    instruction(g, |g| {
        g.add_effect(source, source, EffectKind::LoseAllCreatureTypes, Duration::Indefinite);
        g.add_effect(source, source, EffectKind::AddSubtype(words.subtype(Subtype::Kithkin)), Duration::Indefinite);
        g.add_effect(source, source, EffectKind::AddSubtype(words.subtype(kind)), Duration::Indefinite);
        g.add_effect(source, source, EffectKind::SetPt(power, toughness), Duration::Indefinite);
    });
}

fn become_scout(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        become_kithkin(g, item.object, Subtype::Scout, 2, 3);
    }
    Ok(())
}

/// "If this creature is a Scout": checked as the ability resolves, so it does nothing before the first stage.
fn become_soldier(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) && g.chars(item.object).has_subtype(g.words(item.object).subtype(Subtype::Scout)) {
        become_kithkin(g, item.object, Subtype::Soldier, 4, 5);
    }
    Ok(())
}

/// "If this creature is a Soldier": it becomes an Avatar 7/8 with protection from each of its controller's opponents,
/// the opponents as the ability resolves.
fn become_avatar(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) && g.chars(item.object).has_subtype(g.words(item.object).subtype(Subtype::Soldier)) {
        let opponents: Vec<PlayerId> = g.state().opponents(item.controller).collect();
        instruction(g, |g| {
            become_kithkin(g, item.object, Subtype::Avatar, 7, 8);
            for opponent in opponents {
                g.add_effect(item.object, item.object, EffectKind::ProtectionFromPlayer(opponent), Duration::Indefinite);
            }
        });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{G/W}"), become_scout, "{G/W}: This creature becomes a Kithkin Scout with base power and toughness 2/3."),
        ability(cost("{1}{G/W}{G/W}"), become_soldier, "{1}{G/W}{G/W}: If this creature is a Scout, it becomes a Kithkin Soldier with base power and toughness 4/5."),
        ability(cost("{3}{G/W}{G/W}{G/W}"), become_avatar, "{3}{G/W}{G/W}{G/W}: If this creature is a Soldier, it becomes a Kithkin Avatar with base power and toughness 7/8 and protection from each of your opponents."),
    ],
    ..oracle::FIGURE_OF_FABLE
};
