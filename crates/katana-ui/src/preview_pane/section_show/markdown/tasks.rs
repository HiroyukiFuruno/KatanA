/* WHY: Isolated task list action processing logic to maintain modularity and satisfy file length limits. */

use egui_commonmark::TaskListAction;

pub struct MarkdownTaskOps;

impl MarkdownTaskOps {
    pub fn process_task_list_actions(
        md: &str,
        newly_captured: Vec<TaskListAction>,
        global_task_list_idx: &mut usize,
        actions: &mut Vec<(usize, char)>,
    ) {
        let spans = egui_commonmark::extract_task_list_spans(md);
        for action in newly_captured {
            if let Some(local_idx) = spans.iter().position(|s| s == &action.span) {
                actions.push((*global_task_list_idx + local_idx, action.new_state));
            }
        }
        *global_task_list_idx += spans.len();
    }
}
