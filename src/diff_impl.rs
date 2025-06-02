use chrono::NaiveDate;

pub enum DiffType {
    Status,
    Teacher,
    Room,
    Subject,
    Time,
    ActivityType,
}

pub struct Diffs(pub Vec<DiffType>);

#[derive(Debug, Clone)]
pub enum Diff<'a> {
    AddedNormal(&'a untis::Lesson),
    AddedIrregular(&'a untis::Lesson),
    Changed {
        from: &'a untis::Lesson,
        to: &'a untis::Lesson,
    },
}

impl Diff<'_> {
    pub fn find<'a>(
        prev_schedule: &'a [untis::Lesson],
        new_schedule: &'a [untis::Lesson],
    ) -> Vec<Diff<'a>> {
        let mut diff = Vec::new();

        for new_lesson in new_schedule {
            if let Some(prev_lesson) = prev_schedule.iter().find(|l| l.id == new_lesson.id) {
                if prev_lesson != new_lesson {
                    diff.push(Diff::Changed {
                        from: prev_lesson,
                        to: new_lesson,
                    });
                }
            } else if new_lesson.code != untis::LessonCode::Regular {
                diff.push(Diff::AddedIrregular(new_lesson));
            } else {
                diff.push(Diff::AddedNormal(new_lesson));
            }
        }

        diff
    }

    pub fn date(&self) -> NaiveDate {
        match self {
            Diff::AddedNormal(lesson) => lesson.date.0,
            Diff::AddedIrregular(lesson) => lesson.date.0,
            Diff::Changed { from, .. } => from.date.0,
        }
    }

    pub fn start_time(&self) -> untis::Time {
        match self {
            Diff::AddedNormal(lesson) => lesson.start_time,
            Diff::AddedIrregular(lesson) => lesson.start_time,
            Diff::Changed { from, .. } => from.start_time,
        }
    }
}
