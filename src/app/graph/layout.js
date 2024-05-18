
import React, { useEffect, useState } from 'react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import styles from './layout.module.css';

import { invoke } from "@tauri-apps/api/tauri";


const GraphBody = () => {
 
  const [data, setData] = useState([]);
  const [elapsedTime, setElapsedTime] = useState(0);
  
  useEffect(() => {
    const fetchData = () => {
      invoke("get_current_throughput_wrapper").then((res) => {
        console.log(res);

        // convert res to JSON
        let temp = JSON.parse(res)[0];

        // append current elapsed time to res
        temp.name = elapsedTime
        setElapsedTime(elapsedTime + 5);

        //append data to previous data
        setData([...data, temp]);
        console.log(data);

        // if data records are above 25 remove the last record
        if (data.length > 25) {
          setData(data.slice(1, data.length));
        }

      }).catch((err) => {
        console.log(err);
      });
    };

    fetchData();
    const intervalId = setInterval(fetchData, 5000);
    return () => clearInterval(intervalId);
  });

  return (
    <div className={styles.graphBody}>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart
          width={500}
          height={400}
          data={data}
          margin={{ top: 10, right: 30, left: 0, bottom: 0 }}
          syncId="I"
        >
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="name" />
          <YAxis />
          <Tooltip wrapperClassName={styles.tooltip}/>
          <Area type="monotone" dataKey="up_bps" stackId="1" stroke="#52ff3d" fill="#52ff3d" />
        </AreaChart>
      </ResponsiveContainer>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart
          width={500}
          height={400}
          data={data}
          margin={{ top: 10, right: 30, left: 0, bottom: 0 }}
          syncId="I"
        >
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="name" />
          <YAxis />
          <Tooltip wrapperClassName={styles.tooltip}/>
          <Area type="monotone" dataKey="down_bps" stackId="1" stroke="#5f33ff" fill="#5f33ff" />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
};


const Graph = () => {
  return (
    <div className={styles.graphBody}>
      <GraphBody />
    </div>
  );
};

export default Graph;

