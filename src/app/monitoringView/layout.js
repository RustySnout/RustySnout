'use client'
import React, { useState, useEffect } from 'react';
import Table from "../genericTable/layout";
import Graph from "../graph/layout";
import styles from './layout.module.css';
import { invoke } from "@tauri-apps/api/tauri";
import Image from "next/image";

const MonitoringView = () => {
  const [colHeaders, setColHeaders] = useState([]);
  const [rowData, setRowData] = useState([]);
  const [refresh, setRefresh] = useState(false);

  useEffect(() => {
    const fetchData = () => {
      setRefresh(true);
      invoke("get_process_stats_wrapper").then((res) => {
        console.log(res);
        // parse the json string into cols and rows
        const data = JSON.parse(res);
    
        // set column headers by the Json keys
        setColHeaders(Object.keys(data[0]));
    
        // rows
        setRowData(data);

        setRefresh(false);
      }).catch((err) => {
        console.log(err);
        setRefresh(false);
      });
    };
    
    // Fetch data immediately
    fetchData();
    
    // Then fetch data every 5 seconds (or whatever interval you want)
    const intervalId = setInterval(fetchData, 2000);
    
    // Clear the interval when the component unmounts
    return () => clearInterval(intervalId);
  }, []);
    
  const refreshStyle = refresh ? styles.refreshing : styles.refresh;

  return (
    <div className={styles.monitoringBody}>
      <div className={styles.topContainer}>
        <div className={styles.buttonContainer}>
          <button> 
            <Image src="/process.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/>  
          </button>
          <button> 
            <Image src="/connections.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/>  
          </button>
          <button> 
            <Image src="/remoteAddr.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/>
          </button>
          <button> 
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