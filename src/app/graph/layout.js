
import React, { useEffect, useState } from 'react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import styles from './layout.module.css';
import Image from "next/image";

import { invoke } from "@tauri-apps/api/tauri";

const GraphBody = () => {
 
  const [data, setData] = useState([]);
  const [elapsedTime, setElapsedTime] = useState(0);
  const [warning, setWarning] = useState(false);
  const [updateInterval, setUpdateInterval] = useState(1000);
  
  useEffect(() => {
    const fetchData = async () => {
      invoke("get_current_throughput_wrapper").then((res) => {
        console.log(res);

        // convert res to JSON
        let temp = JSON.parse(res)[0];

        // append current elapsed time to res
        temp.name = elapsedTime
        setElapsedTime(elapsedTime + updateInterval / 1000);

        //append data to previous data
        setData([...data, temp]);
        console.log(data);

        // if data records are above 25 remove the last record
        if (data.length > 25) {
          setData(data.slice(1, data.length));
        }

        // check if the upload and download rates are above 10000
        if (temp.up_bps > 10000 || temp.down_bps > 10000) {
          setWarning(true);
        } else {
          setWarning(false);
        }

      }).catch((err) => {
        console.log(err);
      });
    };

    fetchData();
    const intervalId = setInterval(fetchData, updateInterval);
    return () => clearInterval(intervalId);
  });

  const warningJSX = warning ? 
    <Image className={styles.warning} src="/warning.svg" alt="warning" width={20} height={20} />
   : null;

  return (
    <div className={styles.graphBody}>
      <div className={styles.HeaderContainer}>
        <p>Average Upload and Download Rates</p>
        {warningJSX}
      </div>
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
          <Area type="monotone" dataKey="down_bps" stackId="1" stroke="#2f33ff" fill="#2f33ff" />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
};


const Graph = () => {
  return (
    <div >
      <GraphBody />
    </div>
  );
};

export default Graph;

