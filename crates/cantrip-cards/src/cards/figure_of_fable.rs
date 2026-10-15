use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Each stage replaces the creature's subtypes and base power and toughness (CR 613.1d), where a later effect wins
/// by its timestamp: setting its card types to what they already are clears the subtypes an earlier stage added, and
/// the new ones follow. Figure of Fable is only ever a creature, so its card types do not change.
fn become_kithkin(g: &mut dyn Game, source: ObjectId, kind: Subtype, power: i16, toughness: i16) {
    g.add_effect(source, source, EffectKind::SetTypes(TypeSet::of(&[CardType::Creature])), Duration::Indefinite);
    g.add_effect(source, source, EffectKind::AddSubtype(Subtype::Kithkin), Duration::Indefinite);
    g.add_effect(source, source, EffectKind::AddSubtype(kind), Duration::Indefinite);
    g.add_effect(source, source, EffectKind::SetPt(power, toughness), Duration::Indefinite);
}

fn become_scout(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        become_kithkin(g, item.object, Subtype::Scout, 2, 3);
    }
    Ok(())
}

/// "If this creature is a Scout": checked as the ability resolves, so it does nothing before the first stage.
fn become_soldier(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) && g.chars(item.object).has_subtype(Subtype::Scout) {
        become_kithkin(g, item.object, Subtype::Soldier, 4, 5);
    }
    Ok(())
}

/// "If this creature is a Soldier": it becomes an Avatar 7/8 with protection from each of its controller's opponents,
/// the opponents as the ability resolves.
fn become_avatar(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) && g.chars(item.object).has_subtype(Subtype::Soldier) {
        become_kithkin(g, item.object, Subtype::Avatar, 7, 8);
        let opponents: Vec<PlayerId> = g.state().opponents(item.controller).collect();
        for opponent in opponents {
            g.add_effect(item.object, item.object, EffectKind::ProtectionFromPlayer(opponent), Duration::Indefinite);
        }
    }
    Ok(())
}

const HYBRID: Option<(Mana, Mana)> = Some((Mana::G, Mana::W));

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{G/W}: This creature becomes a Kithkin Scout with base power and toughness 2/3.",
            cost: Cost::mana(ManaCost { hybrid: [HYBRID, None, None, None, None, None], ..mana(0, 0, 0, 0, 0, 0) }),
            effect: become_scout,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}{G/W}{G/W}: If this creature is a Scout, it becomes a Kithkin Soldier with base power and toughness 4/5.",
            cost: Cost::mana(ManaCost { hybrid: [HYBRID, HYBRID, None, None, None, None], ..mana(1, 0, 0, 0, 0, 0) }),
            effect: become_soldier,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{3}{G/W}{G/W}{G/W}: If this creature is a Soldier, it becomes a Kithkin Avatar with base power and toughness 7/8 and protection from each of your opponents.",
            cost: Cost::mana(ManaCost { hybrid: [HYBRID, HYBRID, HYBRID, None, None, None], ..mana(3, 0, 0, 0, 0, 0) }),
            effect: become_avatar,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::FIGURE_OF_FABLE
};
