use std::ffi::OsString;
use std::str::FromStr;
use std::{fs, env};
use std::path::PathBuf;
use std::fs::ReadDir;
use std::thread::sleep;
use std::time::Duration;
use std::process::Command;

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
    println!("SRC: {}",src.display());
    let delays = vec![100,200,500,1000,500,100,200]; //different wait times to try
    for delay in delays
    {
        let result = Command::new("cp")
        .args([&src,&dst])
        .output();
        if result.is_ok()
        {
            return true
        }
        else
        {
            println!("zZz");
            sleep(Duration::from_millis(delay));
        }
    }
    
    false
}

fn get_path_diff(a:&PathBuf,b:&PathBuf) -> Vec<OsString>
{
    let mut count:usize = 0;
    let mut test = b.clone();
    while &test != a
    {
        if test.pop()
        {
            count += 1;
        }
        else
        {
            return Vec::new()
        }
    }
    
    
    let bits:Vec<OsString> = b.iter().map(|c| OsString::from(c)).collect();
    let range:Vec<OsString> = bits[bits.len() - count .. bits.len()].iter().map(|b|b.clone()).collect();
    range
    
}

fn stubborn_copy(src:PathBuf, dst:PathBuf) -> Result<Meta, String>
{
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
        println!("Copying {}",csrc.display());
        let cdst:PathBuf = if csrc != src
        {
            //need to create a new dst
            let new_dirs = get_path_diff(&src,&csrc);
            let mut ndst = dst.clone();
            ndst.extend(new_dirs.iter());
            ndst
        }
        else
        {
            dst.clone()
        };
        
        println!("Target: {}",cdst.display());
        
        if !cdst.exists()
        {
            let mkdirs = fs::create_dir_all(&cdst);
            if mkdirs.is_err()
            {
                meta.dirs_failed += 1;
                continue;
            }
        }
        
        
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

#[cfg(test)]
mod test
{
    use super::*;
    use std::path::PathBuf;

    
    #[test]
    fn test_path_dif()
    {
        let path_a = PathBuf::from(r"/this/dir/");
        
        let path_b = PathBuf::from(r"/this/dir/another");
        
        let path_c = PathBuf::from(r"/this/dir/another/but/i/think/we/good");
        
        let no_match = PathBuf::from(r"/not/related");
        
        let mut test = get_path_diff(&path_a,&path_b);
        
        assert_eq!(test.len(),1);
        assert_eq!(test[0], OsString::from_str("another").unwrap());
        
        test = get_path_diff(&path_a,&path_c);
        
        assert_eq!(test.len(),6);
        assert_eq!(test[0], OsString::from_str("another").unwrap());
        assert_eq!(test[5], OsString::from_str("good").unwrap());
        
        test = get_path_diff(&path_a,&no_match);
        assert_eq!(test.len(),0);
    }
}
