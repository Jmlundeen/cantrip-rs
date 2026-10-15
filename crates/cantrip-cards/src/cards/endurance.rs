use cantrip_api::*;
use cantrip_mechanics::evoke;
use cantrip_mechanics::{targets, triggers::this_enters};
use cantrip_oracle as oracle;

fn green_card(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).colors.has(Color::Green)
}

/// "Up to one target player puts all the cards from their graveyard on the bottom of their library in a random order"
/// (CR 401.4): moved together, shuffled among themselves, and placed under the rest of the library.
fn graveyard_to_bottom(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if item.target_count == 0 || !g.target_legal(item, 0) {
        return Ok(());
    }
    let Target::Player(player) = item.targets[0] else { return Ok(()) };
    let cards: Vec<ObjectId> = g.state().zone(Zone::graveyard(player)).to_vec();
    g.put_into_library(&cards, LibraryPlace::BottomInRandomOrder);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        evoke::sacrifice_trigger("When Endurance enters, if its evoke cost was paid, its controller sacrifices it."),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, up to one target player puts all the cards from their graveyard on the bottom of their library in a random order.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            targets: &[TargetSpec { min: 0, ..targets::PLAYER }],
            effect: graveyard_to_bottom,
            ..TriggeredDef::BLANK
        }),
    ],
    // Evoke: exile a green card from your hand rather than pay this spell's mana cost (CR 118.9).
    ways: &[evoke::way(evoke::exile_from_hand(green_card), "Evoke - Exile a green card from your hand")],
    ..oracle::ENDURANCE
};
