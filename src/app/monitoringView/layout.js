'use client'
import React, { useState, useEffect } from 'react';
import Table from "../genericTable/layout";
import Graph from "../graph/layout";
import styles from './layout.module.css';
import { invoke } from "@tauri-apps/api/tauri";




const MonitoringView = () => {
    const [colHeaders, setColHeaders] = useState([]);
    const [rowData, setRowData] = useState([]);

    useEffect(() => {
        // Define the function that fetches data
        const fetchData = () => {
          invoke("get_process_stats_wrapper").then((res) => {
            console.log(res);
            // parse the json string into cols and rows
            const data = JSON.parse(res);
    
            // set column headers by the Json keys
            setColHeaders(Object.keys(data[0]));
    
            // rows
            setRowData(data);
          }).catch((err) => {
            console.log(err);
          });
        };
    
        // Fetch data immediately
        fetchData();
    
        // Then fetch data every 5 seconds (or whatever interval you want)
        const intervalId = setInterval(fetchData, 5000);
    
        // Clear the interval when the component unmounts
        return () => clearInterval(intervalId);
      }, []);
    

    return (
        <div className={styles.monitoringBody}>
            <Graph />
            <Table rows={rowData} columns={colHeaders} />
        </div>
    );
};

export default MonitoringView;