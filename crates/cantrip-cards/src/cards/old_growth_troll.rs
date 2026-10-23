use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, enduring, targets};
use cantrip_oracle as oracle;

/// "Enchant Forest you control" (CR 702.5a, 303.4): a Forest on the battlefield under the Aura's controller. Forest is a word
/// of this card's rules text (CR 612).
fn forest_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::LAND_YOU_CONTROL.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Forest)))
}

/// A 4/4 green Troll Warrior creature token with trample.
static TROLL_WARRIOR: CardDef = CardDef {
    name: "Troll Warrior",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Green)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Troll, Subtype::Warrior]),
    power: Some(4),
    toughness: Some(4),
    keywords: Keywords::of(&[Keyword::Trample]),
    ..CardDef::BLANK
};

/// "Enchanted Forest has '{T}: Add {G}{G}' and '{1}, {T}, Sacrifice this land: ...'" (CR 303.4, 613.1f): this card's delayed
/// abilities 1 and 2, granted to the Forest the Aura is attached to. A Clone that copied this card and returned as an Aura
/// is another card but was given this card's static ability, so the abilities are always this card's own.
fn grant_forest_abilities(view: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    let card = view.cards().id_of(&CARD);
    for index in [1, 2] {
        if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
            chars.abilities[slot] = Some(AbilityRef::delayed(card, index));
        }
    }
}

/// "Create a tapped 4/4 green Troll Warrior creature token with trample." The Forest is the ability's source (CR 113.7), the
/// token is its controller's.
fn make_troll(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let troll = g.card_id(&TROLL_WARRIOR);
    g.create_tokens(troll, item.controller, 1, true)?;
    Ok(())
}

/// "It's an Aura enchantment with enchant Forest you control and [the static ability below]": what it returns as (CR 614.12),
/// with the first `delayed` ability, which is the static one.
static AS_AURA: EntersAs = EntersAs { aura: true, abilities: &[0], ..EntersAs::NONE };

/// "When Old-Growth Troll dies, if it was a creature, return it to the battlefield. It's an Aura enchantment ...": it comes
/// back under the control of the ability's controller, not its owner (the rulings), and stays in the graveyard when there is
/// no Forest it could enchant (CR 303.4g). A token copy has ceased to exist and is not found there (CR 111.7, 400.7).
fn return_as_aura(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = g.found_after_move(item.object) else { return Ok(()) };
    g.put_onto_battlefield_as(card, item.controller, item, &AS_AURA)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    becomes_aura_enchant: Some(target(forest_you_control)),
    abilities: &[triggered(
        when(EventMask::ZONE_CHANGE, enduring::dies_as_a_creature),
        return_as_aura,
        "When Old-Growth Troll dies, if it was a creature, return it to the battlefield. It's an Aura enchantment with enchant Forest you control and \"Enchanted Forest has '{T}: Add {G}{G}' and '{1}, {T}, Sacrifice this land: Create a tapped 4/4 green Troll Warrior creature token with trample.'\"",
    )],
    delayed: &[
        granted_static_ability(Layer::Ability, attached_object, grant_forest_abilities, "Enchanted Forest has \"{T}: Add {G}{G}\" and \"{1}, {T}, Sacrifice this land: Create a tapped 4/4 green Troll Warrior creature token with trample.\""),
        mana_ability(TAP, add_n(Mana::G, 2), "{T}: Add {G}{G}"),
        ability(cost("{1}, {T}").and(SACRIFICE_THIS), make_troll, "{1}, {T}, Sacrifice this land: Create a tapped 4/4 green Troll Warrior creature token with trample."),
    ],
    ..oracle::OLD_GROWTH_TROLL
};

pub static EXTRA: &[&CardDef] = &[&TROLL_WARRIOR];
