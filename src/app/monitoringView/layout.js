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
  const [refresh, setRefresh] = useState(false);
  const [invokeFunction, setInvokeFunction] = useState("get_process_stats_wrapper");

  //sleep 
  const sleep = (milliseconds) => {
    return new Promise(resolve => setTimeout(resolve, milliseconds))
  }

  const handleProcess = () => {
    sleep(1000);
    setInvokeFunction("get_process_wrapper");
    fetchData();
  };

  const handleConnection = () => { 
    sleep(1000);
    setInvokeFunction("get_process_stats_wrapper");
    fetchData();
  };

  const handleRemoteAddr = () => { 
    sleep(1000);
    setInvokeFunction("get_connections_wrapper");
    fetchData();
  };

  const fetchData = () => {
    setRefresh(true);
    invoke(invokeFunction)
      .then((res) => {
        console.log(res);
        const data = JSON.parse(res);
        setColHeaders(Object.keys(data[0]));
        setRowData(data);
        setRefresh(false);
      })
      .catch((err) => {
        console.log(err);
        setRefresh(false);
      });
  };

  useEffect(() => {
    fetchData();
    const intervalId = setInterval(fetchData, 2000);
    return () => clearInterval(intervalId);
  }, []);

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
