# eventcam-converter

This project provides tools for consuming event camera data files provided by the 
[Event-Camera Dataset and Simulator](http://rpg.ifi.uzh.ch/davis_data.html)
into a fast binary format (flatbuffer) that can be processed without ROS. 


## Source Data Format
The data provided by the [Event Camera Simulator](http://rpg.ifi.uzh.ch/davis_data.html)
and related tools is generally in this format:

- `events.txt`: One timestamped pixel change event per line (timestamp x y polarity). The timestamp is in a 64-bit double float format used by ROS. X and Y are unsigned integer pixel coordinates. Polarity is typically 0 (falling) or 1 (rising).
- `images.txt`: A list of timestamped images, one image reference per line (timestamp filename). Note that there are typically many fewer images than there are events, and that image frames are captured long after the events.
- `images/00000001.png`: The series of images referenced from `images.txt`
- `imu.txt`: One timestamped IMU measurement per line (timestamp ax ay az gx gy gz)
- `groundtruth.txt`: One timestamped ground truth measurements per line (timestamp px py pz qx qy qz qw)
- `calib.txt`: Camera calibration parameters (fx fy cx cy k1 k2 p1 p2 k3)

All of the above text files are CSV, space-delimited, with one item per line. 

This project provides a binary and sample `events.txt` file in `./data/events.text`
