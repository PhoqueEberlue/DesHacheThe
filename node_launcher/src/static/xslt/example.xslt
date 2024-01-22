<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">

    <xsl:template match="/">
        <html>
            <head>
                <title>System Information</title>
            </head>
            <body>
                <h1>System Information</h1>

                <xsl:for-each select="//record">
                    <!-- Start of record -->
                    <h2>CPUs</h2>
                    <ul>
                        <xsl:for-each select=".//cpu">
                            <li>
                                <xsl:value-of select="@name" />: Usage <xsl:value-of select="@usage" />%,
        Frequency <xsl:value-of select="@frequency" /> MHz </li>
                        </xsl:for-each>
                    </ul>

                    <h2>
        RAM</h2>
                    <p>Total: <xsl:value-of select=".//ram/@total" /> bytes</p>
                    <p>Used: <xsl:value-of
                            select=".//ram/@used" /> bytes</p>
                    <p>Total Swap: <xsl:value-of
                            select=".//ram/@total_swap" /> bytes</p>
                    <p>Used Swap: <xsl:value-of
                            select=".//ram/@used_swap" /> bytes</p>

                    <h2>Disks</h2>
                    <ul>
                        <xsl:for-each select=".//disk">
                            <li>
                                <xsl:value-of select="@name" />: Kind <xsl:value-of select="@kind" />,
        File System <xsl:value-of select="@file_system" />, Total Space <xsl:value-of
                                    select="@total_space" /> bytes, Available Space <xsl:value-of
                                    select="@available_space" /> bytes </li>
                        </xsl:for-each>
                    </ul>

                    <h2>
        Networks</h2>
                    <ul>
                        <xsl:for-each select=".//network">
                            <li>
                                <xsl:value-of select="@name" />: Received <xsl:value-of
                                    select="@received" /> bytes, Total Received <xsl:value-of
                                    select="@total_received" /> bytes, Transmitted <xsl:value-of
                                    select="@transmitted" /> bytes, Total Transmitted <xsl:value-of
                                    select="@total_transmitted" /> bytes </li>
                        </xsl:for-each>

                    </ul>
                    <!-- End of record -->
                    <hr /> <!--
                    Separator between records -->
                </xsl:for-each>

            </body>
        </html>
    </xsl:template>

</xsl:stylesheet>