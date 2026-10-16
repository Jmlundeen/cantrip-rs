use cantrip_api::*;
use cantrip_mechanics::attach::equip;
use cantrip_mechanics::for_mirrodin;
use cantrip_oracle as oracle;

/// "Whenever equipped creature and at least one other creature attack": one declaration of two or more attackers with
/// the equipped creature among them (CR 508.1). A creature put onto the battlefield attacking afterwards is no part of it.
fn equipped_and_another_attack(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::AttackersDeclared { count, .. } if *count >= 2 => {
            let equipped = q.attached_to(source)?;
            q.is_attacking(equipped).then_some(Fired::PLAIN).flatten()
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        for_mirrodin::TRIGGER,
        triggered(when(EventMask::ATTACKERS_DECLARED, equipped_and_another_attack), cantrip_mechanics::effects::draw::<1>, "Whenever equipped creature and at least one other creature attack, draw a card."),
        equip(mana(1, 1, 0, 0, 0, 0), "Equip {1}{W}"),
    ],
    ..oracle::GLIMMER_LENS
};

pub static EXTRA: &[&CardDef] = &[&for_mirrodin::REBEL];
