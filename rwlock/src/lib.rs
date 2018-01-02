use std::sync::{Arc, Mutex};
use std::ops::{Deref, DerefMut};
use std::cell::UnsafeCell;
use std::sync::Condvar;
use std::ops::Drop;
use std::rc::Rc;

/// Provides a reader-writer lock to protect data of type `T`
pub struct RwLock<T> {
    data: UnsafeCell<T>,
    pref: Preference,
    order: Order,
    mlock: Mutex<()>,
    operation: UnsafeCell<Operation>,
    dataI: i32,
    dataJ: i32,
    dataK: i32,
    dataL: i32,
    type1: bool,
}

#[derive(PartialEq)]
pub enum Preference {
    /// Readers-preferred
    /// * Readers must wait when a writer is active.
    /// * Writers must wait when a reader is active or waiting, or a writer is active.
    Reader,
    /// Writers-preferred: 
    /// * Readers must wait when a writer is active or waiting.
    /// * Writer must wait when a reader or writer is active.
    Writer,
}

/// In which order to schedule threads
pub enum Order {
    /// First in first out
    Fifo,
    /// Last in first out
    Lifo,
}

pub struct Operation {
	waitingToRead: Vec<Arc<Condvar>>,
	waitingToWrite: Vec<Arc<Condvar>>,
	activeReading: i32, 
	activeWriting: i32,
}

impl<T> RwLock<T> {
    /// Constructs a new `RwLock`
    ///
    /// data: the shared object to be protected by this lock
    /// pref: which preference
    /// order: in which order to wake up the threads waiting on this lock
    pub fn new(data: T, pref: Preference, order: Order) -> RwLock<T> {
        RwLock{
    		mlock: Mutex::new(()),
    		data: UnsafeCell::new(data),
    		pref: pref,
    		order: order,
    		operation: UnsafeCell::new(
    			Operation{
    				waitingToRead: Vec::new(),
    				waitingToWrite: Vec::new(),
    				activeReading: 0,
    				activeWriting: 0,
    		}),
    		dataI: 0,
    		dataJ: 0,
    		dataK: 0,
    		dataL: 0,
    		type1: false,
    	}
    }

    fn waitOnType(&self, input: &str) -> bool {
    	let mut wait = true;
        let  waitingToWrite = unsafe{ &((*self.operation.get()).waitingToWrite) };
        let writeLen = waitingToWrite.len();
        let  activeWriting = unsafe { (*self.operation.get()).activeWriting };
        let waitingToRead = unsafe{ &((*self.operation.get()).waitingToRead) };
        let readLen = waitingToRead.len();
        let activeReading = unsafe{ (*self.operation.get()).activeReading };
        let mut dataI = self.dataI;
        let mut dataJ = self.dataJ;

    	match input{
    		"read" => {
    			match self.pref {
            		Preference::Reader => {
                		if activeWriting > 0{
                			wait = true;
               			 } else {
                			wait = false;
                		}
                		dataI = 1;

            		},
            		Preference::Writer => {

                		if activeWriting > 0 || writeLen > 0{
                			wait = true;
                		} else {
                			wait = false;
                		}
                		dataJ = 2;
           			}
        		}

    		},
    		"write" => {
		        match self.pref {
		            Preference::Reader => {
		            	if activeReading > 0 || readLen > 0 || activeWriting > 0 {
		            		wait = true;
		            	} else {
		            		wait = false;
		            	}
		            	dataI = 1;
		            },
		            Preference::Writer => {
		                if activeWriting > 0 || activeReading > 0{
		                	wait = true;
		                } else {
		                	wait = false;
		                }
		                dataJ = 2;
		            }
		        }
    		},
    		_ => (),
    	}
    	self.checkDatas();
    	return wait;

    }

    /// Requests a read lock, waits when necessary, and wakes up at the earliest opportunity
    /// 
    /// Always returns Ok(_).
    /// (We declare this return type to be `Result` to be compatible with `std::sync::RwLock`)
    pub fn read(&self) -> Result<RwLockReadGuard<T>, ()> {
        let mut guard = self.mlock.lock().unwrap();
        let condVar = Arc::new(Condvar::new()); 
        let mut dataK = self.dataK;
        let mut dataL = self.dataL;
        unsafe {
            (*self.operation.get()).waitingToRead.push(condVar.clone());
        }
        
        while self.waitOnType("read") {
            guard = condVar.wait(guard).unwrap();
        }
        
        unsafe {

            match self.order {
                Order::Fifo => {
                    (*self.operation.get()).waitingToRead.remove(0);
                    dataK = 1;

                },
                Order::Lifo => {
                    (*self.operation.get()).waitingToRead.pop();
                    dataL = 2;
                }
            }
            (*self.operation.get()).activeReading += 1;
        }
        self.checkDatas();

        Ok(
            RwLockReadGuard {
                lock: &self
            }
        )

        
    }

    /// Requests a write lock, and waits when necessary.
    /// When the lock becomes available,
    /// * if `order == Order::Fifo`, wakes up the first thread
    /// * if `order == Order::Lifo`, wakes up the last thread
    /// 
    /// Always returns Ok(_).
    pub fn write(&self) -> Result<RwLockWriteGuard<T>, ()> {

        let mut guard = self.mlock.lock().unwrap();
        let condVar = Arc::new(Condvar::new()); 
        let mut dataK = self.dataK;
        let mut dataL = self.dataL;
        unsafe {
            (*self.operation.get()).waitingToWrite.push(condVar.clone());
        }
        while self.waitOnType("write") {
            guard = condVar.wait(guard).unwrap();
        }
        
        unsafe {
            match self.order {
                Order::Fifo => {
                    (*self.operation.get()).waitingToWrite.remove(0);
                    dataK = 1;
                },
                Order::Lifo => {
                    (*self.operation.get()).waitingToWrite.pop();
                    dataL = 2;
                }
            }   
            (*self.operation.get()).activeWriting += 1;
        }
        self.checkDatas();

        
        Ok(
            RwLockWriteGuard {
                lock: &self
            }
        )

    }

    pub fn checkDatas(&self){
    	let mut boolean = false;
    	match self.dataI{
    		1 => boolean = true,
    		2 => boolean = false,
    		_ => ()
    	}
    	match self.dataJ{
    		1 => boolean = true,
    		2 => boolean = false,
    		_ => ()
    	}
    	match self.dataK{
    		1 => boolean = true,
    		2 => boolean = false,
    		_ => ()
    	}
    	match self.dataL{
    		1 => boolean = true,
    		2 => boolean = false,
    		_ => ()
    	}

    }

    pub fn prefReaderOrderLifo(&self){
		unsafe {
            let ref mut waitingToRead =  (*self.operation.get()).waitingToRead;
            let readLen = waitingToRead.len();
            let ref mut waitingToWrite =  (*self.operation.get()).waitingToWrite;
            let writeLen = waitingToWrite.len();
            if readLen >= 1 {
                for i in 0..readLen {
                	let num = readLen - 1 - i;
                    waitingToRead[num].notify_one();
                }
                
            }else if writeLen >= 1 {
                waitingToWrite[writeLen-1].notify_one();
            }
        }
    }

    pub fn prefReaderOrderFifo(&self){
		unsafe {
            let ref mut waitingToRead =  (*self.operation.get()).waitingToRead;
            let readLen = waitingToRead.len();
            let ref mut waitingToWrite =  (*self.operation.get()).waitingToWrite;
            let writeLen = waitingToWrite.len();
            if readLen >= 1 {
                for i in 0..readLen {
                    waitingToRead[i].notify_one();
                }
                
            }else if writeLen >= 1 {
                waitingToWrite[0].notify_one();
            }
        }
    }

    pub fn prefWriterOrderLifo(&self){
    	unsafe {
            let ref mut waitingToRead =  (*self.operation.get()).waitingToRead;
            let readLen = waitingToRead.len();
            let ref mut waitingToWrite =  (*self.operation.get()).waitingToWrite;
            let writeLen = waitingToWrite.len();
    	    if writeLen >= 1 {
    	        waitingToWrite[writeLen-1].notify_one();
    	        
    	    }else if readLen >= 1 {
    	        for i in 0..readLen {
    	        	let num = readLen - 1 - i;
    	            waitingToRead[num].notify_one();
    	        }                     
    	                 
    	    } 
    	}   	    
    }

    pub fn prefWriterOrderFifo(&self){
    	unsafe {
            let ref mut waitingToRead =  (*self.operation.get()).waitingToRead;
            let readLen = waitingToRead.len();
            let ref mut waitingToWrite =  (*self.operation.get()).waitingToWrite;
            let writeLen = waitingToWrite.len();
    	    if writeLen >= 1 {
    	        waitingToWrite[0].notify_one();
    	        
    	    }else if readLen >= 1 {
    	        for i in 0..readLen {
    	            waitingToRead[i].notify_one();
    	        }  
    	                                    
    	    } 
    	}
    	
    }

    pub fn prefReader(&self){
    	match self.order{
    		Order::Lifo => self.prefReaderOrderLifo(),
    		Order::Fifo => self.prefReaderOrderFifo(),
    	}

    }

    pub fn prefWriter(&self){
    	match self.order{
    		Order::Lifo => self.prefWriterOrderLifo(),
    		Order::Fifo => self.prefWriterOrderFifo(),
    	}
    }

    pub fn preferenceAndOrder(&self) {
    	match self.pref {
    		Preference::Reader => self.prefReader(),
    		Preference::Writer => self.prefWriter(),
    	}
    }
}

/// Declares that it is safe to send and reference `RwLock` between threads safely
unsafe impl<T: Send + Sync> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

/// A read guard for `RwLock`
pub struct RwLockReadGuard<'a, T: 'a> {
    lock: &'a RwLock<T>,
}

/// Provides access to the shared object
impl<'a, T> Deref for RwLockReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe{ &*self.lock.data.get()}
    }
}

/// Releases the read lock
impl<'a, T> Drop for RwLockReadGuard<'a, T> {
        fn drop(&mut self) {
            let guard = self.lock.mlock.lock().unwrap();
            unsafe { 
                if (*self.lock.operation.get()).activeReading > 0 {
                    (*self.lock.operation.get()).activeReading -= 1;
                 }
            self.lock.preferenceAndOrder(); 
            };
    }
}

/// A write guard for `RwLock`
pub struct RwLockWriteGuard<'a, T: 'a> {
    lock: &'a RwLock<T>,
}

// Provides access to the shared object
impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe{&*self.lock.data.get()}
    }
}

/// Provides access to the shared object
impl<'a, T> DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get()}
    }
}

/// Releases the write lock
impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        let guard = self.lock.mlock.lock().unwrap();
        unsafe { if (*self.lock.operation.get()).activeWriting > 0 {
                    (*self.lock.operation.get()).activeWriting -= 1;
                }
            assert_eq!((*self.lock.operation.get()).activeWriting, 0);

            self.lock.preferenceAndOrder(); 
        };
    }
}