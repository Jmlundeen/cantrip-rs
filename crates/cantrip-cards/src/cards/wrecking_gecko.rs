use cantrip_api::*;
use cantrip_mechanics::{pump, trample_until_end_of_turn};
use cantrip_mechanics::triggers::ward;
use cantrip_oracle as oracle;

/// "{6}{G}{G}: This creature gets +4/+4 and gains trample until end of turn." (CR 613.4c, 611.2.)
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    instruction(g, |g| {
        pump(g, item.object, item.object, 4, 4);
        trample_until_end_of_turn(g, item.object, item.object);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward::<2>("Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)"),
        ability(cost("{6}{G}{G}"), grow, "{6}{G}{G}: This creature gets +4/+4 and gains trample until end of turn."),
    ],
    ..oracle::WRECKING_GECKO
};
