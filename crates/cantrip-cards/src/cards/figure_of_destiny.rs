use cantrip_api::*;
use cantrip_oracle as oracle;

/// Each stage sets the creature's creature types and its base power and toughness (CR 205.1a, 613.1d, 613.4b), with no
/// duration (CR 611.2a), so a later activation outranks an earlier one by timestamp (CR 613.6): paying {R/W} again after
/// the Warrior stage makes it a 2/2 Kithkin Spirit again. Figure of Destiny is only ever a creature, so setting its card
/// types to what they are clears the subtypes an earlier stage added, as Figure of Fable's stages do.
fn become_forms(g: &mut dyn Game, source: ObjectId, kinds: &[Subtype], power: i16, toughness: i16) {
    g.add_effect(source, source, EffectKind::SetTypes(TypeSet::of(&[CardType::Creature])), Duration::Indefinite);
    let words = g.words(source);
    g.add_effect(source, source, EffectKind::AddSubtype(words.subtype(Subtype::Kithkin)), Duration::Indefinite);
    for kind in kinds {
        g.add_effect(source, source, EffectKind::AddSubtype(words.subtype(*kind)), Duration::Indefinite);
    }
    g.add_effect(source, source, EffectKind::SetPt(power, toughness), Duration::Indefinite);
}

fn become_spirit(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.is_on_battlefield(item.object) {
        become_forms(g, item.object, &[Subtype::Spirit], 2, 2);
    }
    Ok(())
}

/// "If this creature is a Spirit": checked as the ability resolves (CR 603.4).
fn become_warrior(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.is_on_battlefield(item.object) && g.chars(item.object).has_subtype(g.words(item.object).subtype(Subtype::Spirit)) {
        become_forms(g, item.object, &[Subtype::Spirit, Subtype::Warrior], 4, 4);
    }
    Ok(())
}

/// "If this creature is a Warrior": it becomes an 8/8 Avatar with flying and first strike.
fn become_avatar(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.is_on_battlefield(item.object) && g.chars(item.object).has_subtype(g.words(item.object).subtype(Subtype::Warrior)) {
        become_forms(g, item.object, &[Subtype::Spirit, Subtype::Warrior, Subtype::Avatar], 8, 8);
        g.add_effect(
            item.object,
            item.object,
            EffectKind::AddKeywords(Keywords::of(&[Keyword::Flying, Keyword::FirstStrike])),
            Duration::Indefinite,
        );
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{R/W}"), become_spirit, "{R/W}: This creature becomes a Kithkin Spirit with base power and toughness 2/2."),
        ability(cost("{R/W}{R/W}{R/W}"), become_warrior, "{R/W}{R/W}{R/W}: If this creature is a Spirit, it becomes a Kithkin Spirit Warrior with base power and toughness 4/4."),
        ability(cost("{R/W}{R/W}{R/W}{R/W}{R/W}{R/W}"), become_avatar, "{R/W}{R/W}{R/W}{R/W}{R/W}{R/W}: If this creature is a Warrior, it becomes a Kithkin Spirit Warrior Avatar with base power and toughness 8/8, flying, and first strike."),
    ],
    ..oracle::FIGURE_OF_DESTINY
};
