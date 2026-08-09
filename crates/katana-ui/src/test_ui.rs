pub(crate) struct TestUiOps;

impl TestUiOps {
    pub(crate) fn run(
        context: &egui::Context,
        input: egui::RawInput,
        render: impl FnMut(&mut egui::Ui),
    ) {
        context.run_ui(input, render).drop_without_applying_deltas();
    }
}

#[derive(Default)]
pub(crate) struct Context(egui::Context);

impl Context {
    pub(crate) fn run_ui(
        &self,
        input: egui::RawInput,
        render: impl FnMut(&mut egui::Ui),
    ) -> Output {
        Output(Some(self.0.run_ui(input, render)))
    }
}

impl std::ops::Deref for Context {
    type Target = egui::Context;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) struct Output(Option<egui::FullOutput>);

impl std::ops::Deref for Output {
    type Target = egui::FullOutput;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("test UI output is available")
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        if let Some(output) = self.0.take() {
            output.drop_without_applying_deltas();
        }
    }
}
