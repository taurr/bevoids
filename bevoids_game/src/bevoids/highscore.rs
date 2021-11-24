use bevy::{log, prelude::*};
use chrono::{DateTime, Utc};
use derive_more::{Add, AddAssign, AsRef, Constructor, Display, From, Into};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::bevoids::{settings::Settings, AssetPath};

#[derive(Debug)]
pub(crate) struct AddScoreEvent(pub Score);

#[derive(
    Debug,
    Display,
    Default,
    Clone,
    Copy,
    AsRef,
    Add,
    AddAssign,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Constructor,
    From,
    Into,
    Serialize,
    Deserialize,
)]
pub(crate) struct Score(u32);

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct HighScore {
    score: Score,
    name: String,
    time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct HighScoreRepository {
    scores: Vec<HighScore>,
    max_records: u8,
}

impl HighScoreRepository {
    #[allow(dead_code)]
    pub fn with_capacity(max_records: u8) -> Self {
        Self {
            scores: Vec::with_capacity(max_records as usize),
            max_records,
        }
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = &HighScore> {
        self.scores.iter()
    }

    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.scores.len()
    }

    #[allow(dead_code)]
    pub fn position(&self, score: &Score) -> Option<usize> {
        match self
            .scores
            .iter()
            .enumerate()
            .find(|(_, sr)| *score >= sr.score)
            .map(|(index, _)| index)
        {
            idx @ Some(_) => idx,
            None => {
                if self.scores.is_empty() {
                    Some(0)
                } else if self.max_records as usize > self.scores.len() {
                    Some(self.scores.len())
                } else {
                    None
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn lowest_score(&self) -> Score {
        if let Some(last) = self.scores.last() {
            last.score
        } else {
            Score::new(u32::MIN)
        }
    }

    #[allow(dead_code)]
    pub fn push(&mut self, score: HighScore) -> Result<usize, ()> {
        let index = match self
            .scores
            .iter()
            .enumerate()
            .find(|(_, sr)| score.score >= sr.score)
            .map(|(index, _)| index)
        {
            Some(index) => {
                self.scores.insert(index, score);
                index
            }
            None => {
                self.scores.push(score);
                self.scores.len() - 1
            }
        };

        self.scores.truncate(self.max_records as usize);
        if self.scores.len() > index {
            Ok(index)
        } else {
            Err(())
        }
    }
}

impl HighScore {
    #[allow(dead_code)]
    pub fn new<T: Into<String>>(score: Score, name: T) -> Self {
        Self {
            score,
            name: name.into(),
            time: Utc::now(),
        }
    }

    #[allow(dead_code)]
    pub fn score(&self) -> Score {
        self.score
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &String {
        &self.name
    }

    #[allow(dead_code)]
    pub fn time(&self) -> &DateTime<Utc> {
        &self.time
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use super::*;

    #[test]
    fn highscore_is_send_and_sync() {
        fn assert_send<T: Send + Sync>() {}
        assert_send::<HighScoreRepository>();
    }

    #[test]
    fn default_creates_an_empty_board() {
        let highscores = HighScoreRepository::with_capacity(5);
        assert!(highscores.iter().count() == 0);
    }

    #[quickcheck]
    fn qc_scores_are_sorted_highest_first(count: u8) -> TestResult {
        if count == 0 {
            return TestResult::discard();
        }

        let mut highscores = HighScoreRepository::with_capacity(count);
        for _ in 0..count {
            let _ = highscores.push(HighScore::new(rand::random::<u32>().into(), ""));
        }
        assert!(highscores
            .iter()
            .map(|sr| sr.score)
            .tuple_windows()
            .all(|(a, b)| a >= b));

        assert!(
            matches!(highscores.push(HighScore::new(u32::MAX.into(), "")), Ok(_)),
            "Highest highscore should always succeeed"
        );
        assert!(
            matches!(highscores.push(HighScore::new(u32::MIN.into(), "")), Err(_)),
            "Low scores are not making it to the highscore list"
        );
        assert_eq!(
            highscores.scores.len(),
            count as usize,
            "Number of scores total cannot exceed capacity"
        );
        TestResult::passed()
    }
}

pub(crate) fn update_score(
    mut addscore_events: EventReader<AddScoreEvent>,
    mut score: ResMut<Score>,
) {
    let score_sum = Score::from(
        addscore_events
            .iter()
            .map(|e| -> u32 { u32::from(e.0) })
            .sum::<u32>(),
    );
    if u32::from(score_sum) > 0 {
        *score += score_sum;
        log::info!(
            score = u32::from(score_sum),
            total = u32::from(*score),
            "update score"
        );
    }
}

pub(crate) fn load_highscores(
    mut commands: Commands,
    assets_path: Res<AssetPath>,
    settings: Res<Settings>,
) {
    let pb = highscores_path(&assets_path);
    if let Ok(content) = std::fs::read_to_string(pb.as_path()) {
        let highscores: Result<HighScoreRepository, _> = serde_json::from_str(&content);
        if let Ok(mut highscores) = highscores {
            highscores.scores.sort_by(|h1, h2| h2.score.cmp(&h1.score));
            highscores
                .scores
                .truncate(settings.general.highscores_capacity as usize);
            highscores.max_records = settings.general.highscores_capacity;
            commands.insert_resource(highscores);
            return;
        }
    }
    let highscores = HighScoreRepository::with_capacity(settings.general.highscores_capacity);
    save_highscores(&highscores, &assets_path);
    commands.insert_resource(highscores);
}

fn highscores_path(assets_path: &AssetPath) -> PathBuf {
    let mut pb = PathBuf::from(assets_path.as_str());
    pb.push("highscores.json");
    pb
}

pub(crate) fn save_highscores(highscores: &HighScoreRepository, assets_path: &AssetPath) {
    let pb = highscores_path(assets_path);
    let highscores = serde_json::to_string_pretty(&highscores).unwrap();
    std::fs::write(pb, highscores).expect("unable to write highscores");
}
