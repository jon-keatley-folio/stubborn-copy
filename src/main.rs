use std::str::FromStr;
use std::{fs, env};
use std::path::PathBuf;
use std::fs::ReadDir;
use std::thread::sleep;
use std::time::Duration;

struct Meta
{
    copied:usize,
    failed:usize,
    dirs_created:usize,
    dirs_failed:usize,
}

impl Meta {
    pub fn new() -> Meta
    {
        Meta {
            copied:0,
            failed:0,
            dirs_created:0,
            dirs_failed:0,
        }
    }
}

fn copy_file(src:&PathBuf, dst:&PathBuf) -> bool
{
    let delays = vec![100,200,500,1000,500,100,200]; //different wait times to try
    for delay in delays
    {
        let result = fs::copy(src, dst);
        if result.is_ok()
        {
            return true
        }
        else
        {
            sleep(Duration::from_millis(delay));
        }
    }
    
    false
}

fn stubborn_copy(src:PathBuf, dst:PathBuf) -> Result<Meta, String>
{
    //let dir_result = fs::read_dir("/run/user/1000/gvfs/gphoto2:host=Apple_Inc._iPhone_00008101000A1D3A2145001E");
    let mut meta = Meta::new();
    let mut dirs_to_process:Vec<(PathBuf,ReadDir)> = Vec::new();
    let dir_result = fs::read_dir(&src);
    
    if let Ok(dir) = dir_result
    {
        dirs_to_process.push((src.clone(),dir));
    }
    else
    {
        return Err(String::from("Unable to read this junk!"))
    } 
        
    while let Some((csrc,dir)) = dirs_to_process.pop()
    {
        
        let cdst:PathBuf = if csrc != src
        {
            //need to create a new dst
            dst.clone()
        }
        else
        {
            dst.clone()
        };
        
        
        for entry_result in dir
        {
            if let Ok(entry) = entry_result
            {
                let src_path = entry.path();
                if src_path.is_dir()
                {
                    let can_ls = fs::read_dir(&src_path);
                    if let Ok(ls) = can_ls
                    {
                        dirs_to_process.push((src_path,ls));
                    }
                    else
                    {
                        meta.dirs_failed += 1;
                    }
                }
                else
                {
                    if copy_file(&src_path, &cdst)
                    {
                        meta.copied += 1;
                    }
                    else
                    {
                        meta.failed += 1;
                        //TODO verbose mode?
                    }
                    
                }
            }
            else
            {
                meta.failed += 1;
            }
        }
        
    }
    
    Ok(meta)

}


fn main() 
{
    let args:Vec<String> =  env::args().skip(1).collect();
    if args.len() < 2 || args.len() > 3
    {
        println!("Unexpected usage. stubborn-copy [SRC] [DST]");
        return;
    }
    
    let has_src = PathBuf::from_str(&args[0]);
    let has_dst = PathBuf::from_str(&args[1]);
    #[allow(irrefutable_let_patterns)]
    if let Ok(src) = has_src
    {
        if let Ok(dst) = has_dst
        {
            let results = stubborn_copy(src, dst);
            
            match results
            {
                Ok(meta) => 
                {
                    println!("Success!");
                    println!("Copied: {}",meta.copied);
                    println!("Failed: {}",meta.failed);
                    println!("Directories created: {}",meta.dirs_created);
                    println!("Directories failed: {}",meta.dirs_failed);
                },
                Err(err) => 
                {
                    println!("I failed you :(");
                    println!("Error: {}",err);
                }
            }
        }
        else
        {
            println!("Unable to handle dst! {}", &args[1]);
        }
    }
    else
    {
        println!("Unable to handle src! {}", &args[0]);
    }
    

}
