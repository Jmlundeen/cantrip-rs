use cantrip_api::*;
use cantrip_mechanics::statics::{plus_pt, your_creatures};
use cantrip_mechanics::triggers::creature_you_control_deals_combat_damage_to_player;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

const JESKAI: u8 = 0;
const TEMUR: u8 = 1;

/// "As this enchantment enters, choose Jeskai or Temur" (CR 614.12a, 607.2f).
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    choose::option_as_enters(g, entering, &CARD, 0..2)
}

/// "• Jeskai — Whenever one or more creatures you control deal combat damage to a player" (CR 603.2c): one trigger for each
/// player the batch damaged, and none unless Jeskai was chosen.
fn jeskai_damage(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    if choose::chosen_option(q, me, &CARD) != Some(JESKAI) {
        return None;
    }
    creature_you_control_deals_combat_damage_to_player(q, me, event)
}

/// "Draw a card."
fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

/// "• Temur — Creatures you control ...": your creatures, only while Temur is the word chosen.
fn temur_creatures(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    choose::chosen_option_in_layers(q, source, &CARD) == Some(TEMUR) && your_creatures(q, source, affected, so_far)
}

/// "... have trample and haste" (CR 702.19, 702.10, layer 6).
fn trample_and_haste(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Trample).with(Keyword::Haste);
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    labels: &["Jeskai", "Temur"],
    abilities: &[triggered(
        when_asked(EventMask::DAMAGE, jeskai_damage),
        draw_a_card,
        "Jeskai \u{2014} Whenever one or more creatures you control deal combat damage to a player, draw a card.",
    )
    .sums_per_subject()],
    statics: &[
        static_ability(Layer::Ability, temur_creatures, trample_and_haste, "Temur \u{2014} Creatures you control have trample and haste."),
        static_ability(Layer::PtModify, temur_creatures, plus_pt::<1, 0>, "Temur \u{2014} Creatures you control get +1/+0."),
    ],
    ..oracle::FROSTCLIFF_SIEGE
};
