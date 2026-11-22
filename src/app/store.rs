
use std::rc::Rc;
use slint::{ Model, VecModel };
use crate::TaskForm;
use crate::app::layout::task_height_px;


// this struct stores all tasks in their layouts
#[derive(Clone)]
pub struct Store {
    // a VecModel is slints observable vector
    // Rc so it can be shared between closure captures
    pub left: Rc<VecModel<TaskForm>>,
    pub right: Rc<VecModel<TaskForm>>,
    pub scheduled: Rc<VecModel<TaskForm>>,
}
impl Store {

    // just creates three empty models
    pub fn new() -> Self {
        Self {
            left: Default::default(),
            right: Default::default(),
            scheduled: Default::default(),
        }
    }

    // returns all tasks of a column
    pub fn panel_column(&self, col: i32) -> &Rc<VecModel<TaskForm>> {
        if col == 0 {
            &self.left
        } else {
            &self.right
        }
    }

    // search all three places to find where the task is
    pub fn find(&self, id: i32) -> Option<(&Rc<VecModel<TaskForm>>, usize)> {
        [&self.left, &self.right, &self.scheduled]
            .into_iter()
            // if im not wrong, find_map is the same as filter_map + find
            .find_map(|m| { // for each element m
                m.iter()    // look inside m
                    .position(|t| t.id == id)   // find the index of matching t
                    .map(|i| (m, i))            // if found, create a pair of m and index i
            })
    }
    
    // the panel with shorter column_height
    // used to balance the mansory layout
    pub fn shorter_panel_column(&self, hour_h: f32, min_h: f32) -> &Rc<VecModel<TaskForm>> {
        if column_height(&self.left, hour_h, min_h) <= column_height(&self.right, hour_h, min_h) {
            &self.left
        } else {
            &self.right
        }
    }
    
    // locate a task by id across all columns
    // returns (owning model, index, clone of form)
    pub fn find_task(&self, id: i32)
    -> Option<(&Rc<VecModel<TaskForm>>, usize, TaskForm)> {
        let (m, i) = self.find(id)?;
        Some((m, i, m.row_data(i)?))
    }
}

// sum of all task's task_height_px
pub fn column_height(m: &VecModel<TaskForm>, hour_h: f32, min_h: f32) -> f32 {
    // for each task t, do task_height_px() and sum it up
    m.iter().map(|t| task_height_px(t.duration, hour_h, min_h)).sum()
}

/*
pub fn balance_columns(t: &[TaskForm], hour_h: f32, min_h: f32)
-> (Vec<TaskForm>, Vec<TaskForm>) {
// TODO later
}
*/
