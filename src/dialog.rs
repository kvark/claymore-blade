//! Story beats and short lines. Keep under the 5×7 font budget.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneId {
    RakiJoin,
    RecruitPaburo,
    RecruitPieta,
    OpheliaIntro,
    TownDoga,
    TownDogaLate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneState {
    pub id: SceneId,
    pub step: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Line {
    pub speaker: &'static str,
    pub text: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct Choice {
    pub label: &'static str,
    pub yes: bool,
}

impl SceneState {
    pub fn new(id: SceneId) -> Self {
        Self { id, step: 0 }
    }

    pub fn lines(&self) -> &'static [Line] {
        match self.id {
            SceneId::RakiJoin => RAKI,
            SceneId::RecruitPaburo => RECRUIT_PABURO,
            SceneId::RecruitPieta => RECRUIT_PIETA,
            SceneId::OpheliaIntro => OPHELIA,
            SceneId::TownDoga => TOWN_DOGA,
            SceneId::TownDogaLate => TOWN_DOGA_LATE,
        }
    }

    pub fn current(&self) -> Option<&'static Line> {
        self.lines().get(self.step)
    }

    pub fn at_end(&self) -> bool {
        self.step + 1 >= self.lines().len()
    }

    pub fn advance(&mut self) -> bool {
        if self.step + 1 < self.lines().len() {
            self.step += 1;
            true
        } else {
            false
        }
    }

    pub fn choices(&self) -> &'static [Choice] {
        if !self.at_end() {
            return &[];
        }
        match self.id {
            SceneId::RakiJoin => &[
                Choice {
                    label: "KEEP UP",
                    yes: true,
                },
                Choice {
                    label: "STAY HERE",
                    yes: false,
                },
            ],
            SceneId::RecruitPaburo | SceneId::RecruitPieta => &[
                Choice {
                    label: "WALK WITH US",
                    yes: true,
                },
                Choice {
                    label: "NOT YET",
                    yes: false,
                },
            ],
            SceneId::OpheliaIntro => &[Choice {
                label: "END HER",
                yes: true,
            }],
            SceneId::TownDoga | SceneId::TownDogaLate => &[Choice {
                label: "CONTINUE",
                yes: true,
            }],
        }
    }
}

pub const INTRO: &str = "You are Clare, No. 47.\n\nThe Organization branded you, ranked you,\nand sent you to the island to cut the things\nthat wear human skin.\n\nDoga has lit a beacon.\nWalk there.\nDo not raise the bar unless you must.";

pub const TITLE_FLAVOR: &[&str] = &[
    "Silver eyes see what men will not.",
    "Rank is a number. The bar is a choice.",
    "They call you a Claymore. You are a knife with a name.",
    "Open the bar only when the alternative is dying.",
    "The Organization does not mourn numbers.",
    "Beacons are clocks. The island does not wait.",
];

pub const RESULT_WIN_TITLE: &str = "The board is quiet.";
pub const RESULT_WIN_BODY: &str =
    "You walk back with blood on the silver. The beacon goes dark.";
pub const RESULT_LOSE_TITLE: &str = "You fall.";
pub const RESULT_LOSE_BODY: &str = "The Organization will send another number.";
pub const RESULT_LATE_TITLE: &str = "Too late.";
pub const RESULT_LATE: &str = "You arrive to ash. The nest has moved into the cellars.";

/// Optional combat flavor. Prefer these over pure mechanical text for narrative kinds.
pub fn bark(kind: &str) -> Option<&'static str> {
    match kind {
        "trans" => Some("Clare opens the bar."),
        "trans_high" => Some("The silver in her eyes goes thin."),
        "sever" => Some("The arm learns it is optional."),
        "death_yoma" | "death" => Some("The face stops being a face."),
        "death_silver" => Some("A number goes dark."),
        "raki_drop" => Some("Raki's voice cuts through. The bar falls."),
        "miria_phantom" => Some("Miria is already gone from where they aimed."),
        "helen_stretch" => Some("Helen takes the far hex without stepping."),
        "ophelia_ripple" => Some("The ground forgets who owns it."),
        "raise" => Some("Clare opens the bar."),
        _ => None,
    }
}

pub const TOWN_REST: &str = "Sleep is a low bar. Take it.";
pub const TOWN_LEAVE: &str = "The road does not care about rank.";
pub const TOWN_EMPTY: &str = "No new beacons. The island is holding its breath.";

const RAKI: &[Line] = &[
    Line {
        speaker: "RAKI",
        text: "You killed it. You killed the thing that took them.",
    },
    Line {
        speaker: "RAKI",
        text: "I have nowhere left. I'm coming.",
    },
    Line {
        speaker: "CLARE",
        text: "The road does not care about boys.",
    },
    Line {
        speaker: "RAKI",
        text: "Then I'll die closer to it than here.",
    },
];

const RECRUIT_PABURO: &[Line] = &[
    Line {
        speaker: "MIRIA",
        text: "You cut clean. Most low ranks flinch.",
    },
    Line {
        speaker: "HELEN",
        text: "Rank 47? Cute. Don't trip on that sword.",
    },
    Line {
        speaker: "MIRIA",
        text: "Walk with us until the next nest. Or don't.",
    },
    Line {
        speaker: "MIRIA",
        text: "The Organization will not mourn either choice.",
    },
];

const RECRUIT_PIETA: &[Line] = &[
    Line {
        speaker: "DENEVE",
        text: "I will not die here.",
    },
    Line {
        speaker: "DENEVE",
        text: "That is not a boast. It is a schedule.",
    },
    Line {
        speaker: "DENEVE",
        text: "If the north is a trap, we walk in with open eyes.",
    },
];

const OPHELIA: &[Line] = &[
    Line {
        speaker: "OPHELIA",
        text: "A little sister. How nice.",
    },
    Line {
        speaker: "CLARE",
        text: "I'm here to end you.",
    },
    Line {
        speaker: "OPHELIA",
        text: "That's what friends are for.",
    },
    Line {
        speaker: "OPHELIA",
        text: "Don't run. Friends don't run.",
    },
];

const TOWN_DOGA: &[Line] = &[
    Line {
        speaker: "ELDER",
        text: "You're one of them. Good.",
    },
    Line {
        speaker: "ELDER",
        text: "Something in the well has been wearing faces.",
    },
    Line {
        speaker: "ELDER",
        text: "We lit the beacon three nights ago. Two families are gone.",
    },
];

const TOWN_DOGA_LATE: &[Line] = &[
    Line {
        speaker: "ASH",
        text: "You arrive to ash. The nest has moved into the cellars.",
    },
    Line {
        speaker: "ASH",
        text: "The well is empty of faces. The houses are not.",
    },
    Line {
        speaker: "ASH",
        text: "The boy is gone. No one left to hold your wrist.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raki_has_a_choice_at_the_end() {
        let mut s = SceneState::new(SceneId::RakiJoin);
        while s.advance() {}
        assert!(s.at_end());
        assert_eq!(s.choices().len(), 2);
    }

    #[test]
    fn ophelia_ends_in_a_fight() {
        let mut s = SceneState::new(SceneId::OpheliaIntro);
        while s.advance() {}
        assert_eq!(s.choices()[0].label, "END HER");
    }

    #[test]
    fn bark_covers_raise_and_sever() {
        assert_eq!(bark("trans"), Some("Clare opens the bar."));
        assert_eq!(bark("sever"), Some("The arm learns it is optional."));
        assert!(bark("hit").is_none());
    }
}
