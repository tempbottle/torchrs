#![allow(unused_variables)]
use autograd::{Function, FuncIntf, FuncDelegate, FIWrap};
use tensor::{TensorKindList, OptTensorKindList, TensorKind, NumLimits};

#[builder(pattern="owned")]
#[derive(Builder, Clone)]
pub struct DropoutArgs {
    #[builder(default="0.5")]
    pub p: f64,
    #[builder(default="false")]
    pub training: bool,
    #[builder(default="false")]
    pub inplace: bool,
}

impl Default for DropoutArgs {
    fn default() -> Self {
        DropoutArgsBuilder::default().build().unwrap()
    }
}

impl_func_args!(Dropout1d, DropoutArgs);
impl_func_args!(Dropout2d, DropoutArgs);

trait Noise: FuncIntf {
    fn make_noise(&self, input: &TensorKind) -> TensorKind;
}

trait Dropout: Noise + FuncIntf {
    fn dropout_forward(&mut self,
                       input: &mut TensorKindList,
                       args: &DropoutArgs)
                       -> TensorKindList {
        let mut output = if args.inplace {
            self.mark_dirty(input);
            input[0].clone()
        } else {
            input[0].copy()
        };
        if args.p > 0. && args.training {
            let noise = self.make_noise(&input[0])
                .bernoulli_(1. - args.p)
                .div_(1. - args.p)
                .expand_as(&input[0]);
            output.mult_(&noise);
            vec![noise, output]
        } else {
            vec![output]
        }
    }
    fn dropout_backward(&mut self,
                        grad_output_: &mut OptTensorKindList,
                        args: &DropoutArgs,
                        noise: TensorKind)
                        -> OptTensorKindList {
        match grad_output_[0] {
            None => return vec![None],
            Some(ref mut grad_output) => {
                if args.p > 0. && args.training {
                    vec![Some(grad_output.mult_(&noise).clone())]
                } else {
                    vec![Some(grad_output.clone())]
                }
            }
        }
    }
}

impl Dropout for Dropout1d {}
impl Dropout for Dropout2d {}
impl Noise for Dropout1d {
    fn make_noise(&self, input: &TensorKind) -> TensorKind {
        input.new(()).resize_as_(input)
    }
}
impl Noise for Dropout2d {
    fn make_noise(&self, input: &TensorKind) -> TensorKind {
        let mut v = vec![input.size()[0], input.size()[1]];
        for _ in 0..input.dim() - 2 {
            v.push(1)
        }
        input.new(v)
    }
}
impl FuncIntf for Dropout1d {
    fn forward(&mut self, input: &mut TensorKindList) -> TensorKindList {
        let args = self.args.clone();
        let mut result = self.dropout_forward(input, &args);
        if self.args.p > 0. && args.training {
            self.saved_tensors.push(result.remove(0))
        }
        result
    }
    fn backward(&mut self, grad_output: &mut OptTensorKindList) -> OptTensorKindList {
        let args = self.args.clone();
        println!("dropout1d backward");
        let noise = self.saved_tensors.remove(0);
        self.dropout_backward(grad_output, &args, noise)
    }
}
impl FuncIntf for Dropout2d {
    fn forward(&mut self, input: &mut TensorKindList) -> TensorKindList {
        let args = self.args.clone();
        let mut result = self.dropout_forward(input, &args);
        if self.args.p > 0. && args.training {
            self.saved_tensors.push(result.remove(0))
        }
        result
    }
    fn backward(&mut self, grad_output: &mut OptTensorKindList) -> OptTensorKindList {
        let args = self.args.clone();
        println!("dropout2d backward");
        let noise = self.saved_tensors.remove(0);
        self.dropout_backward(grad_output, &args, noise)
    }
}
