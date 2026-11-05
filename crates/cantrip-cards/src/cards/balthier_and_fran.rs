use cantrip_api::*;
use cantrip_mechanics::{
    history::first_combat_phase_of_the_turn,
    lands,
    statics::{plus_pt, your_of_subtype},
};
use cantrip_oracle as oracle;

const VEHICLE: u16 = lands::sub(Subtype::Vehicle);

const PAY: Question = Question::new("Pay", "pay {1}{R}{G} for an additional combat phase?", "", "Pay {cost}", "Don't pay");

/// "Vehicles you control get +1/+1 and have reach and vigilance": each of the Vehicles its controller controls.
fn your_vehicles(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    your_of_subtype::<VEHICLE>(q, source, affected, so_far)
}

fn have_reach_and_vigilance(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Reach).with(Keyword::Vigilance);
}

/// "Whenever a Vehicle crewed by Balthier and Fran this turn attacks" (CR 508.3a, 702.122c): a Vehicle, whoever controls it, that
/// has been crewed this turn by this creature, asked of the history.
fn vehicle_crewed_by_this_attacks(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let Event::Attacks { attacker, .. } = event else { return None };
    let vehicle = q.words(me.source).subtype(Subtype::Vehicle);
    if q.chars(*attacker).has_subtype(vehicle) && q.history().crewed(*attacker).any(|c| c == me.source) {
        return Fired::PLAIN;
    }
    None
}

/// "You may pay {1}{R}{G}. If you do, after this phase, there is an additional combat phase." (CR 500.8)
fn pay_for_an_additional_combat_phase(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.may_pay(item.controller, item.object, item.object, cost("{1}{R}{G}"), &PAY)? {
        g.add_combat_phase();
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ATTACKS, vehicle_crewed_by_this_attacks),
        pay_for_an_additional_combat_phase,
        "Whenever a Vehicle crewed by Balthier and Fran this turn attacks, if it's the first combat phase of the turn, you may pay {1}{R}{G}. If you do, after this phase, there is an additional combat phase.",
    )
    .condition(first_combat_phase_of_the_turn)],
    statics: &[
        static_ability(Layer::PtModify, your_vehicles, plus_pt::<1, 1>, "Vehicles you control get +1/+1 and have reach and vigilance."),
        static_ability(Layer::Ability, your_vehicles, have_reach_and_vigilance, "Vehicles you control get +1/+1 and have reach and vigilance."),
    ],
    ..oracle::BALTHIER_AND_FRAN
};
