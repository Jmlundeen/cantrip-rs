use cantrip_api::*;
use cantrip_mechanics::statics::{grant_delayed_ability, plus_pt};
use cantrip_mechanics::{attach, targets, tokens};
use cantrip_oracle as oracle;

/// "As long as enchanted permanent is a creature" (CR 604.1, 613.1d): the permanent this Aura enchants, once it is a creature
/// after the layers before this one.
fn enchanted_permanent_if_creature(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    attach::attached_object(q, source, affected, so_far) && so_far.is(CardType::Creature)
}

/// "Channel — {1}{G}, Discard this card: Create a 1/1 green Human Monk creature token with "{T}: Add {G}."" (CR 111.1).
fn create_1_1_green_human_monk(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::HUMAN_MONK, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{1}{G}").and(DISCARD_THIS),
            create_1_1_green_human_monk,
            "Channel — {1}{G}, Discard this card: Create a 1/1 green Human Monk creature token with \"{T}: Add {G}.\"",
        )
        .from(ZoneKind::Hand),
    ],
    statics: &[
        static_ability(
            Layer::Ability,
            enchanted_permanent_if_creature,
            |q, source, chars| {
                chars.keywords = chars.keywords.with(Keyword::Reach);
                grant_delayed_ability(q, source, chars);
            },
            "As long as enchanted permanent is a creature, it gets +1/+3 and has reach and \"{T}: Add {G}{G}.\"",
        ),
        static_ability(
            Layer::PtModify,
            enchanted_permanent_if_creature,
            plus_pt::<1, 3>,
            "As long as enchanted permanent is a creature, it gets +1/+3 and has reach and \"{T}: Add {G}{G}.\"",
        )
        .continues(),
    ],
    delayed: &[mana_ability(TAP, add_n(Mana::G, 2), "{T}: Add {G}{G}.")],
    enchant: Some(targets::ARTIFACT_OR_CREATURE),
    ..oracle::CAREFUL_CULTIVATION
};

pub static EXTRA: &[&CardDef] = &[&tokens::HUMAN_MONK];
