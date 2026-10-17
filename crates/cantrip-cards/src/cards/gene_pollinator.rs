use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Tap an untapped permanent you control" (CR 118.3): any untapped permanent of the ability's controller, a creature that has
/// not been under its control since the start of the turn included (it is not the {T} symbol of that creature, CR 302.6).
/// Never this creature itself: its {T} is the other part of the same cost, and one permanent can't be tapped for both.
fn untapped_permanent_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    id != asker.source && q.controller(id) == asker.controller && !q.obj(id).tapped
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        TAP.and(Cost::tap_untapped(1, untapped_permanent_you_control)),
        add_any_color(1),
        "{T}, Tap an untapped permanent you control: Add one mana of any color.",
    )],
    ..oracle::GENE_POLLINATOR
};
