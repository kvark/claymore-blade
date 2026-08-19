//! Story beats and short lines. Keep under the 5×7 font budget.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneId {
    RakiJoin,
    RecruitPaburo,
    RecruitPieta,
    OpheliaIntro,
    TownDoga,
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
            SceneId::TownDoga => &[Choice {
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
];

pub const RESULT_WIN_TITLE: &str = "The board is quiet.";
pub const RESULT_WIN_BODY: &str =
    "You walk back with blood on the silver. The beacon goes dark.";
pub const RESULT_LOSE_TITLE: &str = "You fall.";
pub const RESULT_LOSE_BODY: &str = "The Organization will send another number.";
pub const RESULT_LATE: &str = "You arrive to ash. The nest has moved into the cellars.";

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
}
