use autograd::{Function, ExecutionEngine, FuncId};
use tensor::Tensor;
use std::ops::{AddAssign, Index};
use std::collections::VecDeque;
use std::marker::PhantomData;
use tensor::*;
use ::*;

thread_local! {
    pub static VAR_TABLE: RefCell<VecDeque<VarKindImpl>> = RefCell::new(VecDeque::new());
}

pub type VarList<T> = Vec<Variable<T>>;
pub type VarKindList = Vec<VarKind>;
pub type RefVarKindList<'a> = Vec<&'a VarKind>;
pub type VarId = i32;
pub enum VarKind {
    FloatVariable(Variable<f32>),
    LongVariable(Variable<i64>),
}
pub enum VarKindImpl {
    FloatVariable(VariableImpl<f32>),
    LongVariable(VariableImpl<i64>),
}

impl<T> From<VarKindImpl> for VariableImpl<T> {
    default fn from(input: VarKindImpl) -> Self {
        unreachable!()
    }
}
impl From<VarKindImpl> for VariableImpl<f32> {
    fn from(input: VarKindImpl) -> Self {
        if let VarKindImpl::FloatVariable(v) = input {
            v
        } else {
            unreachable!()
        }
    }
}
impl From<VarKindImpl> for VariableImpl<i64> {
    fn from(input: VarKindImpl) -> Self {
        if let VarKindImpl::LongVariable(v) = input {
            v
        } else {
            unreachable!()
        }
    }
}

impl<T> From<VariableImpl<T>> for VarKindImpl {
    default fn from(input: VariableImpl<T>) -> Self {
        unreachable!()
    }
}
impl From<VariableImpl<f32>> for VarKindImpl {
    fn from(input: VariableImpl<f32>) -> Self {
        VarKindImpl::FloatVariable(input)
    }
}
impl From<VariableImpl<i64>> for VarKindImpl {
    fn from(input: VariableImpl<i64>) -> Self {
        VarKindImpl::LongVariable(input)
    }
}

impl<T> From<Variable<T>> for VarKind {
    default fn from(input: Variable<T>) -> Self {
        panic!("bad cast")
    }
}
impl From<Variable<f32>> for VarKind {
    fn from(input: Variable<f32>) -> Self {
        VarKind::FloatVariable(input)
    }
}
impl From<Variable<i64>> for VarKind {
    fn from(input: Variable<i64>) -> Self {
        VarKind::LongVariable(input)
    }
}

impl From<VarId> for VarKind {
    fn from(id: VarId) -> VarKind {
        let vecp = VAR_TABLE.with(|f| f.as_ptr());
        let vec = unsafe { &mut *vecp };
        match vec[id as usize] {
            VarKindImpl::FloatVariable(_) => Variable::<f32>::from(id).into(),
            VarKindImpl::LongVariable(_) => Variable::<i64>::from(id).into(),
        }
    }
}

impl<T> From<VarKind> for Variable<T> {
    default fn from(input: VarKind) -> Self {
        panic!("bad cast");
    }
}
impl From<VarKind> for Variable<f32> {
    fn from(input: VarKind) -> Self {
        if let VarKind::FloatVariable(v) = input {
            v
        } else {
            panic!("bad cast")
        }
    }
}
impl From<VarKind> for Variable<i64> {
    fn from(input: VarKind) -> Self {
        if let VarKind::LongVariable(v) = input {
            v
        } else {
            panic!("bad cast")
        }
    }
}
impl<'a, T: 'a> From<&'a VarKind> for &'a Variable<T> {
    default fn from(input: &'a VarKind) -> Self {
        panic!("bad cast");
    }
}
impl<'a> From<&'a VarKind> for &'a Variable<f32> {
    fn from(input: &'a VarKind) -> Self {
        if let &VarKind::FloatVariable(ref v) = input {
            v
        } else {
            panic!("bad cast")
        }
    }
}
impl<'a> From<&'a VarKind> for &'a Variable<i64> {
    fn from(input: &'a VarKind) -> Self {
        if let &VarKind::LongVariable(ref v) = input {
            v
        } else {
            panic!("bad cast")
        }
    }
}

impl Clone for VarKind {
    fn clone(&self) -> Self {
        use self::VarKind::{FloatVariable, LongVariable};
        match *self {
            FloatVariable(ref v) => FloatVariable(v.clone()),
            LongVariable(ref v) => LongVariable(v.clone()),
            _ => unimplemented!(),
        }
    }
}

pub trait VarAccess<T> {
    fn access(&self) -> &mut VariableImpl<T>;
    fn borrow(&self) -> &VariableImpl<T>;
    fn new_args(data: Tensor<T>, args: &VariableArgs) -> Self;
}

impl<T> VarAccess<T> for Variable<T> {
    default fn access(&self) -> &mut VariableImpl<T> {
        panic!("unsupported Tensor type")
    }
    default fn borrow(&self) -> &VariableImpl<T> {
        panic!("unsupported Tensor type")
    }
    default fn new_args(data: Tensor<T>, args: &VariableArgs) -> Self {
        panic!("unsupported Tensor type")
    }
}

impl VarAccess<f32> for Variable<f32> {
    fn access(&self) -> &mut VariableImpl<f32> {
        let vecp = VAR_TABLE.with(|f| f.as_ptr());
        let vec = unsafe { &mut *vecp };
        &mut vec[self.id as usize].into()
    }
    fn borrow(&self) -> &VariableImpl<f32> {
        let vecp = VAR_TABLE.with(|f| f.as_ptr());
        let vec = unsafe { &mut *vecp };
        &vec[self.id as usize].into()
    }
    fn new_args(data: Tensor<f32>, args: &VariableArgs) -> Self {
        let mut id = ::std::usize::MAX;
        let value = VariableImpl::new(data, args);

        VAR_TABLE.with(|f| {
                           let mut table = f.borrow_mut();
                           id = table.len();
                           table.push_back(value.into());
                       });
        Variable {
            id: id as i32,
            phantom: PhantomData,
        }
    }
}

impl VarAccess<i64> for Variable<i64> {
    fn access(&self) -> &mut VariableImpl<i64> {
        let vecp = VAR_TABLE.with(|f| f.as_ptr());
        let vec = unsafe { &mut *vecp };
        &mut vec[self.id as usize].into()
    }
    fn borrow(&self) -> &VariableImpl<i64> {
        let vecp = VAR_TABLE.with(|f| f.as_ptr());
        let vec = unsafe { &mut *vecp };
        &vec[self.id as usize].into()
    }
    fn new_args(data: Tensor<i64>, args: &VariableArgs) -> Self {
        let mut id = ::std::usize::MAX;
        let value = VariableImpl::new(data, args);

        VAR_TABLE.with(|f| {
                           let mut table = f.borrow_mut();
                           id = table.len();
                           table.push_back(value.into());
                       });
        Variable {
            id: id as i32,
            phantom: PhantomData,
        }
    }
}

pub struct VariableImpl<T> {
    pub data: Tensor<T>,
    // AKA Creator Id
    grad_fn: Option<Function>,
    grad: Option<Tensor<T>>,
    // version_counter etc ...
    dirty: bool,
    volatile: bool,
    requires_grad: bool,
}

impl<T> VariableImpl<T> {
    fn new(data_: Tensor<T>, args: &VariableArgs) -> Self {
        let creator = match args.creator {
            Some(ref f) => Some(f.clone()),
            None => None,
        };
        VariableImpl {
            data: data_,
            grad_fn: creator,
            grad: None,
            dirty: false,
            volatile: args.volatile,
            requires_grad: args.requires_grad,
        }
    }
    fn grad(&mut self) -> &mut Tensor<T> {
        let mut output;
        // XXX assert requires_grad
        match self.grad {
            Some(ref mut t) => t,
            None => {
                output = Tensor::new(self.data.size()).zero_();
                self.grad = Some(output);
                self.grad()
            }
        }
    }
    fn _call_hooks(&self, grad_output: T) {
        unimplemented!()
    }
}


pub struct Variable<T> {
    pub id: VarId,
    phantom: PhantomData<T>,
}

impl<T> Default for Variable<T> {
    fn default() -> Self {
        Variable {
            id: -1,
            phantom: PhantomData,
        }
    }
}
impl<T> Clone for Variable<T> {
    fn clone(&self) -> Self {
        Variable {
            id: self.id,
            phantom: PhantomData,
        }
    }
}
impl<T> From<u32> for Variable<T> {
    fn from(id: u32) -> Self {
        Variable {
            id: id as i32,
            phantom: PhantomData,
        }
    }
}
impl<T> From<i32> for Variable<T> {
    fn from(id: i32) -> Self {
        Variable {
            id: id,
            phantom: PhantomData,
        }
    }
}
impl<'a, T: 'a> From<&'a i32> for Variable<T> {
    fn from(id: &'a i32) -> Self {
        Variable {
            id: *id,
            phantom: PhantomData,
        }
    }
}
impl<T> From<usize> for Variable<T> {
    fn from(id: usize) -> Self {
        Variable {
            id: id as i32,
            phantom: PhantomData,
        }
    }
}

#[derive(Default, Clone)]
pub struct BackwardArgs {}

#[derive(Builder)]
#[builder(pattern="owned")]
pub struct VariableArgs {
    pub creator: Option<Function>,
    pub volatile: bool,
    pub requires_grad: bool,
}

impl Default for VariableArgs {
    fn default() -> Self {
        VariableArgs {
            creator: None,
            volatile: false,
            requires_grad: true,
        }
    }
}
impl VariableArgs {
    pub fn build() -> VariableArgsBuilder {
        VariableArgsBuilder::default()
    }
}
impl VariableArgsBuilder {
    pub fn done(self) -> VariableArgs {
        self.build().unwrap()
    }
}

impl VarKind {
    pub fn new_args(data: TensorKind, args: &VariableArgs) -> Self {
        use self::TensorKind::{FloatTensor, LongTensor};
        match data {
            FloatTensor(t) => Variable::<f32>::new_args(t, args).into(),
            LongTensor(t) => Variable::<i64>::new_args(t, args).into(),
            _ => unimplemented!(),
        }
    }

    pub fn is_volatile(&self) -> bool {
        use self::VarKind::{FloatVariable, LongVariable};
        match *self {
            FloatVariable(ref v) => v.is_volatile(),
            LongVariable(ref v) => v.is_volatile(),
            _ => unimplemented!(),
        }
    }
    pub fn varid(&self) -> VarId {
        use self::VarKind::{FloatVariable, LongVariable};
        match *self {
            FloatVariable(ref v) => v.id,
            LongVariable(ref v) => v.id,
            _ => unimplemented!(),
        }
    }
    pub fn requires_grad(&self) -> bool {
        use self::VarKind::{FloatVariable, LongVariable};
        match *self {
            FloatVariable(ref v) => v.requires_grad(),
            LongVariable(ref v) => v.requires_grad(),
            _ => unimplemented!(),
        }
    }
    pub fn grad_fn(&self) -> Option<Function> {
        use self::VarKind::{FloatVariable, LongVariable};
        match *self {
            FloatVariable(ref v) => v.grad_fn(),
            LongVariable(ref v) => v.grad_fn(),
            _ => unimplemented!(),
        }
    }
    pub fn data(&mut self) -> TensorKind {
        use self::VarKind::{FloatVariable, LongVariable};
        match *self {
            FloatVariable(ref v) => v.data().clone().into(),
            LongVariable(ref v) => v.data().clone().into(),
            _ => unimplemented!(),
        }
    }
    pub fn tid(&mut self) -> TensorId {
        use self::VarKind::{FloatVariable, LongVariable};
        match *self {
            FloatVariable(ref v) => v.data().id,
            LongVariable(ref v) => v.data().id,
            _ => unimplemented!(),
        }
    }
    pub fn requires_nograd(&mut self) {
        use self::VarKind::{FloatVariable, LongVariable};
        match *self {
            FloatVariable(ref v) => v.requires_nograd(),
            LongVariable(ref v) => v.requires_nograd(),
            _ => unimplemented!(),
        }
    }
    pub fn typed<T>(self) -> Variable<T> {
        Variable::<T>::from(self)
    }
}

impl<T: Copy> Variable<T> {
    pub fn new(data: Tensor<T>) -> Self {
        Variable::new_args(data, &VariableArgs::default())
    }
    pub fn is_volatile(&self) -> bool {
        self.access().volatile
    }
    pub fn requires_grad(&self) -> bool {
        self.access().requires_grad
    }
    pub fn grad_fn(&self) -> Option<Function> {
        match self.access().grad_fn {
            Some(ref func) => Some(func.clone()),
            None => None,
        }
    }
    pub fn kind(self) -> VarKind {
        VarKind::from(self)
    }
    pub fn data(&mut self) -> &mut Tensor<T> {
        &mut self.access().data
    }
    pub fn data_borrow(&self) -> &Tensor<T> {
        &self.borrow().data
    }
    pub fn apply(&mut self, callback: fn(&mut Tensor<T>)) {
        callback(&mut self.access().data);
    }
    pub fn mark_dirty(&mut self) {
        self.access().dirty = true;
    }
    pub fn requires_nograd(&mut self) {
        self.access().requires_grad = false;
    }
    pub fn view(&self, dims: &[i32]) -> Self {
        unimplemented!()
    }
    // Computes the gradient of current variable w.r.t. graph leaves
    pub fn backward_args(&mut self, gradient_: Option<&mut Tensor<T>>, retain_variables: bool) {
        let mut store;
        let mut gradient;
        {
            let parent = self.access();
            if parent.volatile {
                panic!("calling backward on a volatile variable")
            }
            if !parent.requires_grad {
                panic!("calling backward on a variable that doesn't require a gradient")
            }
            gradient = match gradient_ {
                Some(gradient) => gradient,
                None => {
                    store = parent.data.new_(1);
                    &mut store
                }
            };
        }
        ExecutionEngine::run_backward(self, gradient.clone().into(), retain_variables)
    }
    pub fn _do_backward(&mut self, grad_output: T) {
        let inner = self.access();
        assert_eq!(inner.dirty, false);
        inner._call_hooks(grad_output);
        inner.grad().add_(grad_output);
    }
    pub fn backward(&mut self) {
        self.backward_args(None, false)
    }
    // Detach from graph
    pub fn detach_(&mut self) {
        unimplemented!()
    }
    // return a new variable detached from graph
    pub fn detach(&self) -> Variable<T> {
        unimplemented!()
    }
}

impl<T: Copy> Index<isize> for Variable<T> {
    type Output = T;

    fn index(&self, idx: isize) -> &Self::Output {
        unimplemented!()
    }
}

impl AddAssign<Variable<f32>> for f32 {
    fn add_assign(&mut self, rhs: Variable<f32>) {
        *self = *self + rhs[0]
    }
}
