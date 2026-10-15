use cantrip_api::*;
use cantrip_mechanics::triggers::landfall;
use cantrip_oracle as oracle;

fn mill(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.mill(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Landfall \u{2014} Whenever a land you control enters, mill a card.",
        listens: EventMask::ZONE_CHANGE,
        matches: landfall,
        effect: mill,
        ..TriggeredDef::BLANK
    })],
    // "You may play an additional land on each of your turns" and "You may play lands from your graveyard".
    play_rules: &[PlayRule::ExtraLandPlays(1), PlayRule::LandsFromGraveyard],
    ..oracle::ICETILL_EXPLORER
};
