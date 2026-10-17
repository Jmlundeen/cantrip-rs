use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever this creature transforms into [this face]" (CR 701.27e): the ability is that face's, so it is there to see
/// only once the permanent has turned over to it (CR 603.2, 603.6). A permanent that enters with this face up has not
/// transformed (CR 712.14).
fn this_transforms(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Transformed { object } if *object == source => Fired::PLAIN,
        _ => None,
    }
}

/// "Whenever this creature enters or transforms into Ashling, Rekindled": either event.
fn enters_or_transforms(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    triggers::this_enters(q, source, event).or_else(|| this_transforms(q, source, event))
}

/// "Whenever this creature transforms into Ashling, Rimebound and at the beginning of your first main phase": one
/// ability with two trigger events (CR 603.2).
fn transforms_or_first_main(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    this_transforms(q, source, event).or_else(|| triggers::your_first_main_phase(q, &Asker { source, controller: q.controller(source) }, event))
}

/// "You may discard a card. If you do, draw a card."
fn rummage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::may_discard_to_draw(g, item, 1)
}

/// "You may pay [cost]. If you do, transform Ashling." Asked as the ability resolves, wherever Ashling is: paying is the
/// choice, and Ashling transforms only if it is still the permanent the ability is of and it hasn't transformed since the
/// ability was put onto the stack (CR 701.27f; its ruling). The mana paid is no spell's, so mana for spells can't pay it
/// (CR 106.6).
fn pay_to_transform(g: &mut dyn Game, item: &StackItem, cost: ManaCost) -> Result<(), Suspend> {
    if g.unless_pays(item.controller, item.object, item.object, cost)? {
        g.transform_by(item.object, item);
    }
    Ok(())
}

/// "At the beginning of your first main phase, you may pay {U}. If you do, transform Ashling."
fn pay_blue(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pay_to_transform(g, item, mana("{U}"))
}

/// "At the beginning of your first main phase, you may pay {R}. If you do, transform Ashling."
fn pay_red(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pay_to_transform(g, item, mana("{R}"))
}

/// "Add two mana of any one color. Spend this mana only to cast spells with mana value 4 or greater." Not a mana ability
/// (CR 605.1b: it triggers on no mana ability), so it uses the stack and the color is chosen as it resolves (CR 106.1b).
/// The mana pays only for casting a spell whose mana value is 4 or greater on the stack, X counted (CR 106.6, 202.3e;
/// its ruling), and empties from the pool at the end of the step or phase like any mana (CR 106.4).
fn add_mana(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let color = cantrip_mechanics::choose_color(g, item.controller, item.object)?;
    g.add_restricted_mana(item.controller, color.mana(), 2, ManaRestriction::mana_value_at_least(4));
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask(EventMask::TRANSFORMED.0 | EventMask::STEP_BEGAN.0), transforms_or_first_main), add_mana, "Whenever this creature transforms into Ashling, Rimebound and at the beginning of your first main phase, add two mana of any one color. Spend this mana only to cast spells with mana value 4 or greater."),
        triggered(triggers::YOUR_FIRST_MAIN_PHASE, pay_red, "At the beginning of your first main phase, you may pay {R}. If you do, transform Ashling."),
    ],
    ..oracle::ASHLING_RIMEBOUND
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask(EventMask::ZONE_CHANGE.0 | EventMask::TRANSFORMED.0), enters_or_transforms), rummage, "Whenever this creature enters or transforms into Ashling, Rekindled, you may discard a card. If you do, draw a card."),
        triggered(triggers::YOUR_FIRST_MAIN_PHASE, pay_blue, "At the beginning of your first main phase, you may pay {U}. If you do, transform Ashling."),
    ],
    back: Some(&BACK),
    ..oracle::ASHLING_REKINDLED
};
