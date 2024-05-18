'use client'
import React, { useState, useEffect } from 'react';
import Table from "../genericTable/layout";
import Graph from "../graph/layout";
import styles from './layout.module.css';
import { invoke } from "@tauri-apps/api/tauri";
import Image from "next/image";


// navigate to the control page
const handleControl = () => { };

const MonitoringView = () => {
  const [colHeaders, setColHeaders] = useState([]);
  const [rowData, setRowData] = useState([]);
  const [invokeFunction, setInvokeFunction] = useState("get_process_stats_wrapper");
  const [refresh, setRefresh] = useState(false);  


  const handleProcess = () => {
    setInvokeFunction("get_process_wrapper");
  };

  const handleConnection = () => { 
    setInvokeFunction("get_process_stats_wrapper");
  };

  const handleRemoteAddr = () => { 
    setInvokeFunction("get_connections_wrapper");
  };

  useEffect(() => {
    setRefresh(true);
    const fetchData = () => {
        invoke(invokeFunction).then((res) => {
            setRefresh(false);
            console.log(res);
            const data = JSON.parse(res);
            setColHeaders(Object.keys(data[0]));
            setRowData(data);
        }).catch((err) => {
            setRefresh(false);
            console.log(err);
        });
    };

    fetchData();
    const intervalId = setInterval(fetchData, 5000);
    return () => clearInterval(intervalId);
});

  const refreshStyle = refresh ? styles.refreshing : styles.refresh;

  return (
    <div className={styles.monitoringBody}>
      <div className={styles.topContainer}>
        <div className={styles.buttonContainer}>
          <button onClick={handleProcess}> 
            <Image src="/process.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/>  
          </button>
          <button onClick={handleConnection}> 
            <Image src="/connections.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/>  
          </button>
          <button onClick={handleRemoteAddr}> 
            <Image src="/remoteAddr.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/>
          </button>
          <button onClick={handleControl}> 
            <Image src="/control.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/> 
          </button>
        </div>
        <Graph />
      </div>
      <div className={styles.refreshTable}>
        <Image src="/RustySnout.png" alt="Next.js Logo" width={50} height={50} className={refreshStyle}/>
        <Table rows={rowData} columns={colHeaders} />
      </div>
    </div>
  );
};

export default MonitoringView;
