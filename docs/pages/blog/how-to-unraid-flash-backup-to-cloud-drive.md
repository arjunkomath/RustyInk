---
template: post
title: "How to: Backup UNRAID flash to cloud drive"
author: Arjun Komath
author_link: https://twitter.com/arjunz
date_published: 6 June, 2023
---
Recently while trying to downgrade my UNRAID OS, I completely messed up my server, and the worst part, I did not have a backup of my flash drive. I had to set up my whole server again, lesson learned!

Hence I decided to set up a proper backup solution that would create a flash backup every day, upload it to Google Drive and automatically delete older versions.

## Step 1: Install plugins

You'll need to install the following plugins:

* [**User Scripts**](http://lime-technology.com/forum/index.php?topic=49992.0&ref=techulus.xyz): For running our daily backup script
    
* [**rclone**](http://lime-technology.com/forum/index.php?topic=53365.0&ref=techulus.xyz): For syncing our local folder to cloud drive, in my case, I'm using Google drive.
    

## Step 2: Setup rclone

Setting up rclone should be pretty straightforward, there are plenty of tutorials online, and they have pretty decent documentation also, I followed the instructions here to set up Google Drive, [https://rclone.org/drive/](https://rclone.org/drive/?ref=techulus.xyz)

## Step 3: Create a backup script

We're going to set up a new script by going to Settings -&gt; User Scripts (which can be found under User Utilities). Click 'Add new script' and provide a friendly name. Once the script is created, you should see it show up in the list. Tap on the gear icons and click edit script. Paste the following and save the script.

> Before saving make sure you update the path and name of your Google drive remote that you've set up in the previous step.

```bash
#!/bin/bash

BACKUPDIR=/mnt/user/Chonky/backupFlash

mkdir -p ${BACKUPDIR}

tar -C /boot -zvcf ${BACKUPDIR}/hostname-flash-date +%Y%m%d%H%M%S.tgz --exclude config/super.dat --exclude 'previous*' --exclude 'bz*' --exclude "System Volume Information" .

#cleanup - leave upto 14days worth of backups
find ${BACKUPDIR} -ctime +14 -delete

#backup
rclone sync ${BACKUPDIR} google-drive:/UNRAID-flash-backup --verbose
```

We will go through the script line by line.

`#!/bin/bash` is explained here [https://medium.com/@codingmaths/bin-bash-what-exactly-is-this-95fc8db817bf](https://medium.com/@codingmaths/bin-bash-what-exactly-is-this-95fc8db817bf?ref=techulus.xyz)

Next, we assign a variable `BACKUPDIR` which is the path to our backup folder, this is a path on your UNRAID share. `mkdir` creates the directory.

The `tar` command creates our flash backup archive. Next, we remove archives older than 14 days and finally, the **last line is the command to sync the backup folder to Google drive using rclone.**

## Step 4: Setup script schedule

The last step is to make sure our script runs every day, so go back to user scripts, next to the new script we created you should see a dropdown that says 'Schedule disabled'. Change that to schedule daily and we're done!
